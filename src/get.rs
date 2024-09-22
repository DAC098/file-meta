use std::cmp::{PartialOrd, Ordering};
use std::collections::BinaryHeap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};

use crate::logging;
use crate::tags;
use crate::path;
use crate::db::{self, Db, FileData, MetaContainer};

#[derive(Debug, Eq, Ord)]
enum FilterKey<'a> {
    Borrowed(&'a str),
    Owned(Box<str>),
}

impl<'a> PartialEq for FilterKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FilterKey::Borrowed(a), FilterKey::Borrowed(b)) => a == b,
            (FilterKey::Borrowed(a), FilterKey::Owned(b)) => **a == **b,
            (FilterKey::Owned(a), FilterKey::Borrowed(b)) => **a == **b,
            (FilterKey::Owned(a), FilterKey::Owned(b)) => a == b,
        }
    }
}

impl<'a> PartialOrd for FilterKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (FilterKey::Borrowed(a), FilterKey::Borrowed(b)) => Some(a.cmp(b)),
            (FilterKey::Borrowed(a), FilterKey::Owned(b)) => Some((**a).cmp(& *b)),
            (FilterKey::Owned(a), FilterKey::Borrowed(b)) => Some((**a).cmp(*b)),
            (FilterKey::Owned(a), FilterKey::Owned(b)) => Some(a.cmp(b))
        }
    }
}

impl<'a> Display for FilterKey<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            FilterKey::Borrowed(v) => write!(f, "@ {v}"),
            FilterKey::Owned(v) => write!(f, "@ {v}"),
        }
    }
}

type FilteredList<'a> = Vec<(
    FilterKey<'a>,
    &'a (dyn MetaContainer)
)>;

#[derive(Debug, Clone, ValueEnum)]
enum SortBy {
    Name,
    Date,
    Created,
    Updated,
}

#[derive(Debug, Args)]
pub struct GetArgs {
    /// does not output tags for files
    #[arg(long, conflicts_with("no_comment"))]
    no_tags: bool,

    /// does not output comments for files
    #[arg(long, conflicts_with("no_tags"))]
    no_comment: bool,

    /// retrieves all known data in the db
    #[arg(long)]
    all: bool,

    /// retrieves data from the db itself
    #[arg(long = "self")]
    self_: bool,

    /// sort by created or updated date
    ///
    /// sorting will be done in ascending order. if the order of a value cannot
    /// be determined and there is no other constraint then the order will be
    /// unspecified
    #[arg(long, value_delimiter(','), default_value("name"))]
    sort_by: Vec<SortBy>,

    /// filters out results that do not contain the desired tags
    ///
    /// this will be considered a AND operation with exclude tags, so a given
    /// record must fulfill both include and exclude rules.
    #[arg(long, value_delimiter(','))]
    includes_tags: Vec<tags::TagKey>,

    /// filters out results that do contain the desired tags
    ///
    /// this will be considered a AND operation with include tags, so a given
    /// record must fulfill both include and exclude rules.
    #[arg(long, value_delimiter(','))]
    excludes_tags: Vec<tags::TagKey>,

    /// the file(s) to retrieve data for
    #[arg(
        trailing_var_arg(true),
        default_value("./")
    )]
    files: Vec<PathBuf>,
}

pub fn get_data(args: GetArgs) -> anyhow::Result<()> {
    let context = db::Context::cwd_load()?;

    let mut filtered_items: FilteredList = Vec::new();

    if (args.self_ || args.all) && check_filter(&context.db, &args) {
        filtered_items.push((FilterKey::Borrowed("!SELF"), &context.db));
    }

    if args.all {
        for (key, file) in &context.db.files {
            if !check_filter(file, &args) {
                continue;
            }

            sorted_insert(FilterKey::Borrowed(key), file, &mut filtered_items, &args.sort_by);
        }
    } else {
        for path_result in context.rel_to_db_list(&args.files) {
            let Some((_path, db_entry, existing)) = get_path_data(path_result, &context.db) else {
                continue;
            };

            if !check_filter(existing, &args) {
                continue;
            }

            sorted_insert(FilterKey::Owned(db_entry), existing, &mut filtered_items, &args.sort_by);
        }
    }

    let total = filtered_items.len();
    let print_title = total > 1;

    for (key, data) in filtered_items {
        print_data(&key, data, &args, print_title);
    }

    println!("Total: {total}");

    Ok(())
}

fn check_filter<M>(meta: &M, args: &GetArgs) -> bool
where
    M: MetaContainer
{
    for check in &args.includes_tags {
        if !meta.tags().contains_key(check.inner()) {
            return false;
        }
    }

    for check in &args.excludes_tags {
        if meta.tags().contains_key(check.inner()) {
            return false;
        }
    }

    true
}

fn sorted_insert<'a, M>(key: FilterKey<'a>, meta: &'a M, filtered_items: &mut FilteredList<'a>, sort_by: &[SortBy])
where
    M: MetaContainer,
{
    let result = filtered_items.binary_search_by(|other| {
        for by in sort_by {
            match by {
                SortBy::Name => match other.0.cmp(&key) {
                    Ordering::Equal => {},
                    order => return order,
                }
                SortBy::Date => match other.1.modified().cmp(meta.modified()) {
                    Ordering::Equal => {},
                    order => return order,
                }
                SortBy::Created => match other.1.created().cmp(meta.created()) {
                    Ordering::Equal => {},
                    order => return order,
                }
                SortBy::Updated => match (other.1.updated(), meta.updated()) {
                    (Some(other_updated), Some(meta_updated)) => match other_updated.cmp(meta_updated) {
                        Ordering::Equal => {},
                        order => return order,
                    }
                    (Some(_), None) => return Ordering::Less,
                    (None, Some(_)) => return Ordering::Greater,
                    (None, None) => {}
                }
            }
        }

        Ordering::Equal
    });

    match result {
        Ok(index) => filtered_items.insert(index, (key, meta)),
        Err(index) => filtered_items.insert(index, (key, meta)),
    }
}

fn get_path_data<'a>(
    path_result: Result<path::RelativePath, path::PathError>,
    db: &'a Db,
) -> Option<(Box<Path>, Box<str>, &'a FileData)> {
    let Some(rel_path) = logging::log_result(path_result) else {
        return None;
    };

    let (path, db_entry) = rel_path.into();

    let Some(existing) = db.files.get(&db_entry) else {
        println!("\"{db_entry}\" not found");
        return None;
    };

    Some((path, db_entry, existing))
}

fn print_data<E, M>(entry: &E, container: &M, args: &GetArgs, print_title: bool)
where
    M: MetaContainer + ?Sized,
    E: Display + ?Sized,
{
    let mut printed_key = false;
    let mut print_ts = false;

    if !args.no_tags {
        if print_title {
            println!("{entry}");
            printed_key = true;
        }

        print_tags(container.tags());
        print_ts = true;
    }

    if !args.no_comment {
        if let Some(comment) = container.comment() {
            if print_title && !printed_key {
                println!("{entry}");
            }

            println!("comment: {comment}");
            print_ts = true;
        }
    }

    if print_ts {
        let local_offset = chrono::Local;

        if let Some(updated) = container.updated() {
            let local_updated = updated.with_timezone(&local_offset);

            println!("{local_updated}");
        } else {
            let local_created = container.created()
                .with_timezone(&local_offset);

            println!("{local_created}");
        }
    }
}

fn print_tags(tags: &tags::TagsMap) {
    let mut max_len = 0usize;
    let mut no_value = BinaryHeap::new();
    let mut with_value = BinaryHeap::new();

    for (key, value) in tags {
        if value.is_some() {
            with_value.push(key.clone());

            let chars_count = key.chars().count();

            if chars_count > max_len {
                max_len = key.chars().count();
            }
        } else {
            no_value.push(key.clone());
        }
    }

    for key in no_value.into_sorted_vec() {
        println!("{key}");
    }

    for key in with_value.into_sorted_vec() {
        let value = tags.get(&key)
            .unwrap()
            .as_ref()
            .unwrap();

        println!("{key:>max_len$}: {value}");
    }
}
