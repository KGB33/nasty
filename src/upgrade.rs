use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, fs};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Prints update information on a NixOS system
pub fn nixos(file_path: &str) {
    // Read Nix lockfile
    let contents = read_lockfile(file_path);
    let roots: Vec<&String> = match contents.nodes.get("root") {
        Some(n) => match n {
            NodeType::Root { inputs } => inputs.keys().collect(),
            NodeType::Node { .. } => {
                println!(r#"[{{'error': 'Unable to find inputs on root node'}}]"#);
                return;
            }
        },
        None => {
            println!(r#"[{{'error': 'Unable to find root node'}}]"#);
            return;
        }
    };

    // Check for updates
    let mut updates = HashMap::<String, Update>::new();
    for (_, node) in contents
        .nodes
        .iter()
        .filter(|&(name, _)| roots.contains(&name))
    {
        let (orig, lock) = match node {
            NodeType::Node {
                original, locked, ..
            } => (original, locked),
            NodeType::Root { .. } => continue,
        };
        let checker = checker_factory(lock, orig);
        updates.insert(format!("{}/{}", orig.owner, orig.repo), checker.check());
    }

    // Display Updates
    println!(
        "{}",
        serde_json::to_string(&updates).unwrap_or("{{\"error\": \"idk man\"}".to_string())
    );
}

trait Checker {
    fn check(&self) -> Update;
}

fn checker_factory<'a>(lock: &'a Locked, orig: &'a Original) -> impl Checker + 'a {
    match lock._type {
        SourceType::GitHub => Github {
            use_cli: Github::gh_cli_is_ready(),
            lock,
            orig,
        },
        _ => panic!("No Checker avalable for {:?}", lock._type),
    }
}

struct Github<'a> {
    use_cli: bool,
    lock: &'a Locked,
    orig: &'a Original,
}

impl Checker for Github<'_> {
    fn check(&self) -> Update {
        if self.use_cli {
            return self.check_using_cli();
        }
        self.check_using_http()
    }
}

impl Github<'_> {
    /// Checks to see if the GitHub CLI is prestent and authenticated.
    fn gh_cli_is_ready() -> bool {
        let mut cmd = Command::new("gh");
        let cmd = cmd.arg("auth").arg("status");
        let output = match cmd.output() {
            Ok(o) => o,
            Err(_) => return false, // GH CLI isn't callable
        };
        output.status.code() == Some(0)
    }

    fn generate_enpoint(&self) -> String {
        // /repos/{onwer}/{repo}/compare/{}...{_ref/HEAD}
        // If the _ref isn't incluseed replace with HEAD
        let _ref = self.orig._ref.as_deref().unwrap_or("HEAD");
        format!(
            "repos/{}/{}/compare/{}...{}",
            self.orig.owner, self.orig.repo, self.lock.rev, _ref
        )
    }

    fn check_using_cli(&self) -> Update {
        let endpoint = self.generate_enpoint();
        let mut cmd = Command::new("gh");
        let cmd = cmd.arg("api").arg(endpoint);
        let out = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                return Update::Error {
                    error: e.to_string(),
                }
            }
        };
        if out.status.code() != Some(0) {
            return Update::Error {
                error: String::from_utf8(out.stderr)
                    .unwrap_or("Command failed, and unwrapping srderr failled too!".to_string()),
            };
        };
        let response = match String::from_utf8(out.stdout) {
            Ok(r) => r,
            Err(e) => {
                return Update::Error {
                    error: e.to_string(),
                }
            }
        };
        serde_json::from_str::<Update>(&response).unwrap_or(Update::Error {
            error: String::from("Falled to parse response to json."),
        })
    }

    fn check_using_http(&self) -> Update {
        let source_url = format!("https://api.github.com/{}", self.generate_enpoint());
        let client = match reqwest::blocking::ClientBuilder::new()
            .user_agent("KGB33/nasty")
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return Update::Error {
                    error: e.to_string(),
                }
            }
        };
        let resp = match client.get(source_url).send() {
            Ok(resp) => resp,
            Err(e) => {
                return Update::Error {
                    error: e.to_string(),
                }
            }
        };
        if resp.status() == StatusCode::FORBIDDEN {
            // x-ratelimit-reset
            let ratelimit_reset: i64 = match resp.headers().get("x-ratelimit-reset") {
            Some(r) => match r.to_str() {
                Ok(s) => match s.parse::<i64>() {
                    Ok(i) => i,
                    Err(e) => {
                        return Update::Error {
                            error: e.to_string(),
                        }
                    }
                },
                Err(e) => {
                    return Update::Error {
                        error: e.to_string(),
                    }
                }
            },
            None => return Update::Error {
                error:
                    "You appear to be ratelimited but the 'x-ratelimit-reset' header isn't present."
                        .to_string(),
            },
        };
            let time_til_unlimited = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as i64;
            return Update::Error {
                error: format!(
                    "Rate limited... try again in {}s",
                    ratelimit_reset - time_til_unlimited
                ),
            };
        };
        match resp.json::<Update>() {
            Ok(r) => r,
            Err(e) => Update::Error {
                error: e.to_string(),
            },
        }
    }
}

fn read_lockfile(path: &str) -> Lockfile {
    let contents = fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str::<Lockfile>(&contents).expect("Unable to parse file.")
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum Update {
    Valid { html_url: String, ahead_by: u16 },
    Error { error: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Lockfile {
    nodes: HashMap<String, NodeType>,
    root: String,
    version: u16,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum NodeType {
    Root {
        inputs: HashMap<String, String>,
    },
    Node {
        inputs: Option<HashMap<String, InputType>>,
        original: Original,
        locked: Locked,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum InputType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Locked {
    last_modified: u64,
    nar_hash: String,
    owner: String,
    repo: String,
    rev: String,
    #[serde(alias = "type")]
    _type: SourceType,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Original {
    owner: String,
    #[serde(alias = "ref")]
    _ref: Option<String>,
    repo: String,
    #[serde(alias = "type")]
    _type: SourceType,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
/// See https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html#types
enum SourceType {
    GitHub,
    GitLab,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_url_formats_correctly_ref_some() {
        let orig = &Original {
            owner: "nixos".to_string(),
            _ref: Some("some_ref".to_string()),
            repo: "nixpkgs".to_string(),
            _type: SourceType::GitHub,
        };
        let lock = &Locked {
            last_modified: 12345,
            nar_hash: "some_hash".to_string(),
            owner: "nixos".to_string(),
            repo: "nixpkgs".to_string(),
            rev: "some_rev".to_string(),
            _type: SourceType::GitHub,
        };
        let gh = Github {
            use_cli: false,
            lock,
            orig,
        };
        let expected = "repos/nixos/nixpkgs/compare/some_rev...some_ref".to_string();
        assert_eq!(gh.generate_enpoint(), expected);
    }

    #[test]
    fn github_url_formats_correctly_ref_none() {
        let orig = &Original {
            owner: "nixos".to_string(),
            _ref: None,
            repo: "nixpkgs".to_string(),
            _type: SourceType::GitHub,
        };
        let lock = &Locked {
            last_modified: 12345,
            nar_hash: "some_hash".to_string(),
            owner: "nixos".to_string(),
            repo: "nixpkgs".to_string(),
            rev: "some_rev".to_string(),
            _type: SourceType::GitHub,
        };
        let gh = Github {
            use_cli: false,
            lock,
            orig,
        };
        let expected = "repos/nixos/nixpkgs/compare/some_rev...HEAD".to_string();
        assert_eq!(gh.generate_enpoint(), expected);
    }
    #[test]
    fn root_inputs_are_parsed_correctly() {
        let expected = HashMap::from([
            ("home-manager".to_string(), "home-manager".to_string()),
            ("hyprland".to_string(), "hyprland".to_string()),
            ("nixpkgs".to_string(), "nixpkgs".to_string()),
        ]);
        let lf: Lockfile = serde_json::from_str(
            r#"
            {
                "nodes": {
                    "root": {
                        "inputs": {
                            "home-manager": "home-manager",
                            "hyprland": "hyprland",
                            "nixpkgs": "nixpkgs"
                        }
                    }
                },
                "root": "root",
                "version": 7
            }"#,
        )
        .expect("Failed to parse Root node into HashMap.");
        let actual = match lf.nodes.get("root").unwrap() {
            NodeType::Root { inputs } => inputs,
            _ => panic!("Should only parse to a root node."),
        };
        assert_eq!(&expected, actual);
    }

    #[test]
    fn input_nodes_are_parsed_correctly() {
        let lf: Lockfile = serde_json::from_str(
            r#"
            {
              "nodes": {
                "nixpkgs": {
                  "locked": {
                    "lastModified": 1690640159,
                    "narHash": "sha256-5DZUYnkeMOsVb/eqPYb9zns5YsnQXRJRC8Xx/nPMcno=",
                    "owner": "nixos",
                    "repo": "nixpkgs",
                    "rev": "e6ab46982debeab9831236869539a507f670a129",
                    "type": "github"
                  },
                  "original": {
                    "owner": "nixos",
                    "ref": "nixos-unstable",
                    "repo": "nixpkgs",
                    "type": "github"
                  }
                }
              },
              "root": "root",
              "version": 7
            }
        "#,
        )
        .expect("Failed to parse lockfile.");
        let (actual_locked, actual_original) = match lf.nodes.get("nixpkgs").unwrap() {
            NodeType::Node {
                locked, original, ..
            } => (locked, original),
            _ => panic!("Should only parse to an input node."),
        };
        let expected_locked = Locked {
            last_modified: 1690640159,
            nar_hash: "sha256-5DZUYnkeMOsVb/eqPYb9zns5YsnQXRJRC8Xx/nPMcno=".to_string(),
            owner: "nixos".to_string(),
            repo: "nixpkgs".to_string(),
            rev: "e6ab46982debeab9831236869539a507f670a129".to_string(),
            _type: SourceType::GitHub,
        };
        let expected_original = Original {
            owner: "nixos".to_string(),
            _ref: Some("nixos-unstable".to_string()),
            repo: "nixpkgs".to_string(),
            _type: SourceType::GitHub,
        };
        assert_eq!(&expected_locked, actual_locked);
        assert_eq!(&expected_original, actual_original);
    }

    #[test]
    fn locked_is_parsed_correctly() {
        let actual: Locked = serde_json::from_str(
            r#"
            {
                "lastModified": 1690640159,
                "narHash": "sha256-5DZUYnkeMOsVb/eqPYb9zns5YsnQXRJRC8Xx/nPMcno=",
                "owner": "nixos",
                "repo": "nixpkgs",
                "rev": "e6ab46982debeab9831236869539a507f670a129",
                "type": "github"
             }
        "#,
        )
        .unwrap();
        let expected = Locked {
            last_modified: 1690640159,
            nar_hash: "sha256-5DZUYnkeMOsVb/eqPYb9zns5YsnQXRJRC8Xx/nPMcno=".to_string(),
            owner: "nixos".to_string(),
            repo: "nixpkgs".to_string(),
            rev: "e6ab46982debeab9831236869539a507f670a129".to_string(),
            _type: SourceType::GitHub,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    /// '.nodes.xdph` wasn't parsing, not sure why
    fn ensure_xdph_is_parsed_correctly() {
        let actual: Lockfile = serde_json::from_str(
            r#"
            {
              "root": "root",
              "version": 7,
              "nodes": {
                "xdph": {
                  "inputs": {
                    "hyprland-protocols": [
                      "hyprland",
                      "hyprland-protocols"
                    ],
                    "nixpkgs": [
                      "hyprland",
                      "nixpkgs"
                    ]
                  },
                  "locked": {
                    "lastModified": 1685385764,
                    "narHash": "sha256-r+XMyOoRXq+hlfjayb+fyi9kq2JK48TrwuNIAXqlj7U=",
                    "owner": "hyprwm",
                    "repo": "xdg-desktop-portal-hyprland",
                    "rev": "4d9ff0c17716936e0b5ca577a39e263633901ed1",
                    "type": "github"
                  },
                  "original": {
                    "owner": "hyprwm",
                    "repo": "xdg-desktop-portal-hyprland",
                    "type": "github"
                  }
                }
              }
            }
        "#,
        )
        .expect("Failed to parse json");

        let inputs = HashMap::from([
            (
                "hyprland-protocols".to_string(),
                InputType::Multiple(vec![
                    "hyprland".to_string(),
                    "hyprland-protocols".to_string(),
                ]),
            ),
            (
                "nixpkgs".to_string(),
                InputType::Multiple(vec!["hyprland".to_string(), "nixpkgs".to_string()]),
            ),
        ]);
        let node = NodeType::Node {
            inputs: Some(inputs),
            original: Original {
                owner: "hyprwm".to_string(),
                _ref: None,
                repo: "xdg-desktop-portal-hyprland".to_string(),
                _type: SourceType::GitHub,
            },
            locked: Locked {
                last_modified: 1685385764,
                nar_hash: "sha256-r+XMyOoRXq+hlfjayb+fyi9kq2JK48TrwuNIAXqlj7U=".to_string(),
                owner: "hyprwm".to_string(),
                repo: "xdg-desktop-portal-hyprland".to_string(),
                rev: "4d9ff0c17716936e0b5ca577a39e263633901ed1".to_string(),
                _type: SourceType::GitHub,
            },
        };
        let expected = Lockfile {
            nodes: HashMap::from([("xdph".to_string(), node)]),
            root: "root".to_string(),
            version: 7,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn node_that_is_not_flake() {
        let actual: Lockfile = serde_json::from_str(
            r#"
            {
              "root": "root",
              "version": 7,
              "nodes": {
                "wlroots": {
                  "flake": false,
                  "locked": {
                    "host": "gitlab.freedesktop.org",
                    "lastModified": 1690165843,
                    "narHash": "sha256-gv5kjss6REeQG0BmvK2gTx7jHLRdCnP25po6It6I6N8=",
                    "owner": "wlroots",
                    "repo": "wlroots",
                    "rev": "e8d545a9770a2473db32e0a0bfa757b05d2af4f3",
                    "type": "gitlab"
                  },
                  "original": {
                    "host": "gitlab.freedesktop.org",
                    "owner": "wlroots",
                    "repo": "wlroots",
                    "rev": "e8d545a9770a2473db32e0a0bfa757b05d2af4f3",
                    "type": "gitlab"
                  }
                }
              }
            }
        "#,
        )
        .expect("Failed to parse json");
        let expected = Lockfile {
            nodes: HashMap::from([(
                "wlroots".to_string(),
                NodeType::Node {
                    inputs: None,
                    original: Original {
                        owner: "wlroots".to_string(),
                        _ref: None,
                        repo: "wlroots".to_string(),
                        _type: SourceType::GitLab,
                    },
                    locked: Locked {
                        last_modified: 1690165843,
                        nar_hash: "sha256-gv5kjss6REeQG0BmvK2gTx7jHLRdCnP25po6It6I6N8=".to_string(),
                        owner: "wlroots".to_string(),
                        repo: "wlroots".to_string(),
                        rev: "e8d545a9770a2473db32e0a0bfa757b05d2af4f3".to_string(),
                        _type: SourceType::GitLab,
                    },
                },
            )]),
            root: "root".to_string(),
            version: 7,
        };
        assert_eq!(expected, actual)
    }
}
