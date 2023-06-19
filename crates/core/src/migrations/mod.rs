use sea_query::TableAlterStatement;
use semver::{BuildMetadata, Prerelease, Version};

pub struct Migration {
    pub version: Version,
    pub queries: Vec<TableAlterStatement>,
}

pub struct Migrations(Vec<Migration>);

impl Default for Migrations {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrations {
    fn new() -> Self {
        let mut migrations = Self(Vec::new());

        migrations.add(
            Version {
                major: 0,
                minor: 1,
                patch: 0,
                pre: Prerelease::EMPTY,
                build: BuildMetadata::EMPTY,
            },
            vec![],
        );

        migrations
    }

    fn add(&mut self, version: Version, queries: Vec<TableAlterStatement>) -> &mut Self {
        self.0.push(Migration { version, queries });
        self
    }

    pub fn all_from(version: &str) -> Vec<Migration> {
        let migrations = Self::new();

        let mut targets = migrations
            .0
            .into_iter()
            .filter(|m| m.version > Version::parse(version).unwrap())
            .collect::<Vec<_>>();
        targets.sort_by(|a, b| a.version.cmp(&b.version));

        targets
    }
}
