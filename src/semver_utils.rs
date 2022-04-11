use regex::Regex;
use semver::{Version, Prerelease, BuildMetadata};
use version_compare::{compare_to, Cmp};
use std::cmp::Ordering;
use std::collections::HashMap;

fn validate_version(version: Version) -> Option<Version> {
    if version.pre == Prerelease::EMPTY {
        Some(version)
    } else {
        None
    }
}

fn build_version(version: &Version) -> String {
    format!("{}.{}.{}", version.major, version.minor, version.patch)
}

pub fn get_major_versions(versions: Vec<String>) -> Vec<String> {
    let reg = Regex::new(r"^v").unwrap();

    let mut versions = versions
        .iter()
        .map(|v| reg.replace(&v, ""))
        .collect::<Vec<_>>();

    versions.sort_by(|a, b| {
        if compare_to(a, b, Cmp::Ge).unwrap() == true {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    println!("{:?}", versions);

    let versions = versions
            .iter()
            .filter_map(|v| match Version::parse(&reg.replace(&v, "")).ok() {
                Some(version) => validate_version(version),
                _ => None,
            });
    let mut versionsMap: HashMap<u64, Version> = HashMap::new();
    for version in versions {
        let contain = versionsMap.contains_key(&version.major);
        if !contain {
            versionsMap.insert(version.major, version);
        }
    }
    versionsMap
        .values()
        .map(|v| build_version(&v))
        .collect::<Vec<String>>();
}
