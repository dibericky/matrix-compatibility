use std::{cmp::Ordering, collections::HashMap};

use regex::Regex;
use semver::{Prerelease, Version};
use version_compare::{Cmp, compare_to};


fn validate_version(version: Version) -> Option<Version> {
    if version.pre == Prerelease::EMPTY {
        Some(version)
    } else {
        None
    }
}

fn sort_desc (versions: &mut Vec<String>) {
    versions.sort_by(|a, b| {
        if compare_to(a, b, Cmp::Ge).unwrap_or(false) == true {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    })
}

pub fn get_major_versions(versions: &Vec<String>) -> Vec<String> {
    let reg = Regex::new(r"^v").unwrap();

    let mut versions = versions.to_owned();

    sort_desc(&mut versions);

    let versions = versions
            .iter()
            .filter_map(|v| match Version::parse(&reg.replace(&v, "")).ok() {
                Some(version) => if let Some(valid_version) = validate_version(version) {
                    Some((v, valid_version))
                } else { None },
                _ => None,
            });
    let mut versions_map: HashMap<u64, String> = HashMap::new();
    for (v, version) in versions {
        let contain = versions_map.contains_key(&version.major);
        if !contain {
            versions_map.insert(version.major, v.to_owned());
        }
    }
    let mut majors = versions_map
        .values()
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();
    sort_desc(&mut majors);
    majors
}