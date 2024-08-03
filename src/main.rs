#![allow(clippy::multiple_crate_versions)]

use cargo_metadata::MetadataCommand;
use chrono::Utc;
use colored::Colorize;
use git2::{
    BranchType, Branches, Commit, Diff, DiffFormat, DiffOptions, DiffStats, Index, Repository,
    Revwalk, Status, StatusOptions, Statuses,
};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::env::consts::OS;
use std::fs::{self, read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::path::MAIN_SEPARATOR_STR;
use std::process::{Command, ExitCode};
use walkdir::WalkDir;

const COMMIT_TEMPLATE: &str = "%type%(%scope%): %summary%\n\n\tThe following changes were made :\n\n%why%\n\n%footer%\n\n\tAuthored by :\n\n\t\t* %author% <%email%> the %date%\n";
const CRATES_PATH: &str = "CRATES_PATH";
const INIT: &str = "Init flow";
const COMMIT: &str = "Add a commit";
const STASH: &str = "Stash modification";
const QUIT: &str = "Quit";
const SEND_TO_REMOTE: &str = "Send modifications to remotes";
const PULL: &str = "Get modifications from the remote";
const LIST_PULL_REQUESTS: &str = "Display of your pull request";
const RENAME_REPOSITORY: &str = "Rename the repository";
const CREATE_PULL_REQUEST: &str = "Create a pull request";
const UPDATE_PULL_REQUEST_BRANCH: &str = "Update a pull request branch";
const LOGIN: &str = "Login to GitHub";
const LOGOUT: &str = "Logout to GitHub";
const REFRESH_CREDENTIALS: &str = "Refresh crendentials";
const BROWSE: &str = "Open repository on default browser";
const SHOW_BRANCHES: &str = "Show branches";
const CLONE_GIST: &str = "Clone a gist";
const CLONE_REPO: &str = "Clone a repository";
const CREATE_GIST: &str = "Create a gist";
const EDIT_GIST: &str = "Edit a gist";
const LIST_GIST: &str = "Display your gists";
const VIEW_GIST: &str = "View a gists";
const REMOVE_GIST: &str = "Remove a gist";
const RENAME_GIST: &str = "Rename a file in a gist";
const RELEASE_CREATE: &str = "Create a new release";
const RELEASE_DOWNLOAD: &str = "Download a release";
const RELEASE_EDIT: &str = "Edit a release";
const RELEASE_LIST: &str = "Display releases";
const RELEASE_UPLOAD: &str = "Upload assets to a releases";
const RELEASE_VIEW: &str = "View information about a release";
const RELEASE_DELETE_ASSETS: &str = "Remove an asset from a release";
const CREATE_ISSUES: &str = "Create a issue";
const EDIT_ISSUES: &str = "Edit issues";
const LIST_ISSUES: &str = "Display issues";
const PIN_ISSUES: &str = "Pin an issues";
const UN_PIN_ISSUES: &str = "Unpin an issue";
const REOPEN_ISSUES: &str = "Reopen issue";
const STATUS_ISSUES: &str = "Show status of a relevant issues";
const TRANSFER_ISSUE: &str = "Transfer issue to another repository";
const VIEW_ISSUES: &str = "Display an issue";
const CLOSE_ISSUE: &str = "Close a issue";
const LOCK_ISSUE: &str = "Lock issue conversation";
const UNLOCK_ISSUE: &str = "Unlock issue conversation";
const COMMENT_ISSUE: &str = "Add a comment to an issue";
const GPG_KEY: &str = "Manage gpg key";
const REMOVE_BRANCHES: &str = "Remove branches";
const SHOW_DIFF: &str = "Display diff";
const SHOW_LOGS: &str = "Display logs";
const CANCEL_WORKFLOW: &str = "Cancel a workflow";
const REMOVE_WORKFLOW: &str = "Remove a workflow";
const DOWNLOAD_WORKFLOW: &str = "Download artifacts generated by a workflow run";
const LIST_WORKFLOW: &str = "Display recent workflows run";
const WATCH_WORKFLOW: &str = "Watch a run until it completes, showing its progress";
const VIEW_WORKFLOW: &str = "View a summary of a workflow run";
const RERUN_WORKFLOW: &str = "Rerun a run";
const DELETE_WORKFLOW_CACHE: &str = "Remove github actions caches";
const LIST_WORKFLOW_CACHE: &str = "Display github actions caches";
const GENERATE_CHANGE_LOG: &str = "Generate or update the changelog";
const SHOW_STATUS: &str = "Display workflow status";
const START_FEATURE: &str = "Start a new feature";
const REMOVE_FEATURE: &str = "Remove a feature";
const FINISH_FEATURE: &str = "Finnish a feature";
const START_HOTFIX: &str = "Start a new hotfix";
const REMOVE_HOTFIX: &str = "Remove a hotfix";
const FINISH_HOTFIX: &str = "Finnish a hotfix";
const START_REALEASE: &str = "Start a new release";
const REMOVE_RELEASE: &str = "Remove a release";
const FINISH_RELEASE: &str = "Finnish a release";
const PULL_RELEASE: &str = "Fetch a release";
const PULL_HOTFIX: &str = "Fetch a hotfix";

const OPTIONS: [&str; 69] = [
    INIT,
    COMMIT,
    STASH,
    QUIT,
    SEND_TO_REMOTE,
    PULL,
    LIST_PULL_REQUESTS,
    UPDATE_PULL_REQUEST_BRANCH,
    CREATE_PULL_REQUEST,
    LOGIN,
    LOGOUT,
    REFRESH_CREDENTIALS,
    BROWSE,
    SHOW_BRANCHES,
    CLONE_GIST,
    CLONE_REPO,
    CREATE_GIST,
    LIST_GIST,
    VIEW_GIST,
    EDIT_GIST,
    UNLOCK_ISSUE,
    PULL_HOTFIX,
    PULL_RELEASE,
    REMOVE_GIST,
    CREATE_ISSUES,
    RELEASE_CREATE,
    RELEASE_DELETE_ASSETS,
    RELEASE_EDIT,
    RELEASE_LIST,
    RELEASE_DOWNLOAD,
    RELEASE_UPLOAD,
    RELEASE_VIEW,
    PIN_ISSUES,
    LOCK_ISSUE,
    CLOSE_ISSUE,
    UN_PIN_ISSUES,
    EDIT_ISSUES,
    LIST_ISSUES,
    REOPEN_ISSUES,
    STATUS_ISSUES,
    COMMENT_ISSUE,
    TRANSFER_ISSUE,
    RENAME_REPOSITORY,
    RENAME_GIST,
    GPG_KEY,
    REMOVE_BRANCHES,
    SHOW_LOGS,
    SHOW_DIFF,
    SHOW_STATUS,
    LIST_WORKFLOW,
    CANCEL_WORKFLOW,
    WATCH_WORKFLOW,
    RERUN_WORKFLOW,
    VIEW_WORKFLOW,
    DELETE_WORKFLOW_CACHE,
    REMOVE_WORKFLOW,
    DOWNLOAD_WORKFLOW,
    LIST_WORKFLOW_CACHE,
    REMOVE_FEATURE,
    START_FEATURE,
    FINISH_FEATURE,
    REMOVE_HOTFIX,
    START_HOTFIX,
    FINISH_HOTFIX,
    START_REALEASE,
    FINISH_RELEASE,
    REMOVE_RELEASE,
    VIEW_ISSUES,
    GENERATE_CHANGE_LOG,
];

const CHECK_COMMIT_FILE: &str = "zen";

const LANG: &str = "en_US";

const COMMITS_TYPES: [&str; 68] = [
    "Star: New feature or enhancement",
    "Comet: Bug fix or error resolution",
    "Nebula: Code refactoring",
    "Pulsar: Performance improvement",
    "Quasar: Documentation or clarity improvement",
    "Asteroid Belt: Code cleanup and maintenance",
    "Solar Flare: Testing-related changes",
    "Dwarf Planet: Minor updates or fixes",
    "Terraform: Infrastructure changes",
    "Black Hole: Removing large chunks of code or features",
    "Wormhole: Merging branches or connecting code parts",
    "Big Bang: Initial commit or major feature start",
    "Launch: Deploying to production or releasing a version",
    "Lightspeed: Significant performance improvements",
    "Mission Control: Project management changes",
    "Spacewalk: Urgent hotfixes",
    "Moon Landing: Major milestone or goal completion",
    "First Contact: Initial integrations with external systems",
    "Interstellar Communication: Improving documentation or communication",
    "Solar Eclipse: Temporarily masking functionality",
    "Supernova: Major, transformative change",
    "Meteor Shower: Series of small changes or fixes",
    "Solar Wind: Refactoring code structure",
    "Lunar Eclipse: Temporarily disabling a feature",
    "Cosmic Dawn: Initial implementation of a feature",
    "Solar Storm: Rapid, impactful changes",
    "Lunar Transit: Minor, temporary change",
    "Perihelion: Brings the project closer to its goals or objectives",
    "Aphelion: Immediate goals, but is necessary for long-term progress",
    "White Dwarf: Improving code comments or documentation",
    "Red Giant: Expanding a feature or functionality",
    "Neutron Star: Optimizing code for performance",
    "Binary Star: Merging features or components",
    "Brown Dwarf: Undeveloped feature with potential",
    "Quark Star: Experimental or speculative change",
    "Rogue Planet: Independent change",
    "Stellar Nursery: Creation of new components",
    "Planetary Nebula: Removal or deprecation of a component",
    "Globular Cluster: Collection of related changes",
    "Void: Removal of a module, component, or feature",
    "Gravity: Resolving merge conflicts or dependencies",
    "Dark Matter: Fixing unknown or mysterious bugs",
    "Time Dilation: Improving code performance",
    "Spacetime: Changes to date, time, or scheduling",
    "Gravitational Lensing: Altering data or information flow",
    "Cosmic String: Connecting code parts",
    "Quantum Fluctuation: Small, random change",
    "Hawking Radiation: Removing technical debt",
    "Quantum Entanglement: Establishing close relationships between code parts",
    "Gravitational Redshift: Slowing down or reducing code performance",
    "Space Probe: Testing new features or technologies",
    "Station: Creating or improving environments",
    "Rocket Launch: Deploying to production",
    "Spacewalk: Urgent production hotfixes",
    "Space Elevator: Making codebase more accessible",
    "Warp Drive: Significant speed improvement",
    "Dyson Sphere: Comprehensive optimization of a specific area",
    "Generation Ship: Long-term project for a self -sustaining system",
    "Lagrange Point: Stabilizing or balancing code parts",
    "Orbital Maneuver: Changing project direction",
    "Mission Control: Represents project management-related changes",
    "Moon Landing: Celebrates the completion of major milestones",
    "Interstellar Travel: Migration to a new architecture or language",
    "Rover: Exploration of new technologies or approaches",
    "Singularity: Resolution of a complex or hard-to-reproduce issue",
    "Relativity: Changes related to time, dates, or timestamps",
    "Expansion: Scaling up the system or increasing capacity",
    "Big Crunch: Reduction of codebase size or removal of features",
];

fn check_commit(sentence: &str) -> bool {
    let mut f: File = File::create(CHECK_COMMIT_FILE).expect("msg");
    writeln!(f, "{sentence}").expect("msg");
    let o = Command::new("hunspell")
        .arg("-d")
        .arg(LANG)
        .arg("-l")
        .arg(CHECK_COMMIT_FILE)
        .output()
        .expect("msg")
        .stdout;
    if o.is_empty() {
        return true;
    }
    arrange_commit()
}
fn print_diff(diff: &Diff<'_>) -> Result<(), git2::Error> {
    let stats: DiffStats = diff.stats().expect("msg");
    let x = diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        let content: String = String::from_utf8_lossy(line.content()).into_owned();
        match origin {
            '-' => print!("{} {}", "-".red(), content.red()),
            '+' => print!("{} {}", "+".green(), content.green()),
            '@' => print!("  {}", content.cyan()),
            _ => print!("  {content}"),
        }
        true // Continue iterating
    });

    print!(
        "\n  {} files changed, {} insertions(+), {} deletion(-)\n",
        stats.files_changed() - 1,
        stats.insertions(),
        stats.deletions(),
    );
    x
}
fn diff(path: &str) -> bool {
    let repo: Repository = open(path);
    let mut opts: DiffOptions = DiffOptions::new();
    let changes: Diff<'_> = repo
        .diff_index_to_workdir(
            None,
            Some(&mut opts.include_untracked(true).recurse_untracked_dirs(true)),
        )
        .expect("msg");
    assert!(print_diff(&changes).is_ok());

    true
}
fn add(path: &str) -> Option<Index> {
    let repo: Repository = open(path);
    let statuses: Statuses<'_> = repo.statuses(None).expect("Failed to get status");
    let mut file_options: Vec<String> = Vec::new();
    for entry in &statuses {
        let status: Status = entry.status();
        if status.is_wt_new() || status.is_wt_modified() {
            let path = entry.path().unwrap_or_default();
            file_options.push(path.to_string());
        }
    }
    if file_options.is_empty() {
        println!("No files to add.");
        None
    } else {
        let selected_files: Vec<String> = MultiSelect::new("Select files to add:", file_options)
            .prompt()
            .expect("Fail");
        let mut index = repo.index().expect("msg");
        for file in &selected_files {
            index.add_path(file.as_ref()).expect("msg");
        }
        index.write().expect("msg");

        println!("Added {} files to the index.", selected_files.len());
        Some(index)
    }
}

fn msg(m: &str, r: &str) -> bool {
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(m)
        .current_dir(r)
        .spawn()
        .expect("git")
        .wait()
        .unwrap()
        .success()
}
fn commit(path: &str) -> bool {
    assert!(diff(path));
    let index = add(path);
    if index.is_none() {
        return false;
    }
    msg(
        COMMIT_TEMPLATE
            .replace("%type%", get_commit_types().as_str())
            .replace("%scope%", get_scope().as_str())
            .replace("%summary%", get_summary().as_str())
            .replace("%why%", get_why().as_str())
            .replace("%footer%", get_footer().as_str())
            .replace("%date%", Utc::now().date_naive().to_string().as_str())
            .replace("%author%", name().as_str())
            .replace("%email%", email().as_str())
            .as_str(),
        path,
    )
}
fn arrange_commit() -> bool {
    let _ = Command::new("hunspell")
        .arg("-d")
        .arg(LANG)
        .arg(CHECK_COMMIT_FILE)
        .spawn()
        .expect("Missing dic")
        .wait()
        .unwrap()
        .success();
    check_commit(
        read_to_string(CHECK_COMMIT_FILE)
            .expect("failed to parse zen file")
            .as_str(),
    )
}

fn program_or_lib() -> String {
    if read_to_string("Cargo.toml")
        .expect("no cargo project")
        .contains("lib")
    {
        String::from("library")
    } else {
        String::from("software")
    }
}
fn create_changelog() -> bool {
    if Path::new("./logs").is_dir().eq(&false) {
        fs::create_dir_all("./logs").expect("msg");
    }
    let filename = format!(
        "./logs{MAIN_SEPARATOR_STR}{}-{}-changes.md",
        project(),
        version()
    );
    let mut f: File = File::create(filename.as_str()).expect("failed to create file");
    writeln!(
        f,
        "# 🚀 {} {} released\n\nToday the `{}`, we are very happy to present the **{}** version of our `{}` {} !\n\nThis release marks a significant step forward for our {} {}.\n\n## Demonstration\n\n{}\n\n## What it's?\n\nIt's {}\n\n## What we do ?\n\n- {}\n\n## Our team\n\n- {}\n\n## Links\n\n- [Source code]({})\n- [Home]({})\n- [Issues]({})\n- [Pull Request]({})\n- [Discussions]({})\n- [Wiki]({})\n- [Projects]({})\n- [Releases]({})\n- [Crates.io](https://crates.io/crates/{}/{})\n",
        project(),
        version(),
        Utc::now().date_naive(),
        version(),
        project(),
        program_or_lib(),
        program_or_lib(),
        project(),
        project(),
        description(),
        keywords().join("\n- "),
        authors().join("\n- "),
        repository(),
        homepage(),
        issues(),
        pulls_request(),
        discussions(),
        wiki(),
        projects(),
        releases(),
        project(),
        version()
    )
    .expect("msg");
    let repo: Repository = open(".");
    let mut revwalk = repo.revwalk().expect("msg");
    revwalk.push_head().expect("msg");
    for oid in revwalk {
        let oid = oid.expect("msg");
        let commit = repo.find_commit(oid).expect("msg");
        let message = commit.message().unwrap_or_default();
        let relevant_lines: Vec<&str> = message.lines().collect();
        for l in relevant_lines {
            let line = l.trim();
            if line.is_empty() {
                continue;
            }
            if line.contains('(') {
                writeln!(f, "- {line}").expect("msg");
            }
            if line.contains("The following changes were made :") {
                writeln!(f, "\t- {line}").expect("msg");
            }
            if line.contains('*') {
                writeln!(f, "\t\t- {}", line.to_string().replace('*', "").trim()).expect("msg");
            }
            if line.contains("by") {
                writeln!(f, "\t- {}", line.to_string().replace('*', "").trim()).expect("msg");
            }
            if line.contains('@') {
                writeln!(f, "\t\t- {}", line.to_string().replace('*', "").trim()).expect("msg");
            }
            if line.contains('#') {
                writeln!(f, "\t\t- {}", line.to_string().replace('*', "").trim()).expect("msg");
            }
        }
    }
    writeln!(
        f,
        "\n\n{}\n\n```\n{}\n```\n",
        read_to_string(readme())
            .expect("readme file not founded")
            .trim()
            .replace('#', "##"),
        read_to_string(license())
            .expect("LICENSE file not founded")
            .trim()
    )
    .expect("msg");
    Path::new(filename.as_str()).exists()
}

fn issues() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/issues");
    } else if x.contains("gitlab") {
        x.push_str("-/issues");
    }
    x
}

fn wiki() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/wiki");
    } else if x.contains("gitlab") {
        x.push_str("-/wikis");
    }
    x
}
fn projects() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/projects");
    }
    x
}

fn pulls_request() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/pulls");
    } else if x.contains("gitlab") {
        x.push_str("-/merge_requests");
    }
    x
}

fn discussions() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/discussions");
    }
    x
}

fn fmt() {
    assert!(Command::new("cargo")
        .arg("fmt")
        .current_dir(".")
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());
    clear();
}

fn zuu() -> bool {
    clear();
    if Path::new("Cargo.toml").exists() {
        fmt();
        if Command::new("zuu")
            .current_dir(".")
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success()
        {
            clear();
            return true;
        }
        return false;
    }
    clear();
    true
}

fn version() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package = metadata.packages.first().unwrap();
    package.version.to_string()
}

fn releases() -> String {
    let mut x = repository();
    if x.contains("github") {
        x.push_str("/releases");
    } else if x.contains("gitlab") {
        x.push_str("-/tags");
    }
    x
}
fn project() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package.name.to_string()
}

fn keywords() -> Vec<String> {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package.keywords.clone()
}

fn homepage() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package.clone().homepage.expect("no homepage")
}

fn readme() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package
        .clone()
        .readme
        .expect("no readme define")
        .to_string()
}

fn license() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package
        .clone()
        .license_file
        .expect("no licences define")
        .to_string()
}

fn repository() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package.clone().repository.expect("no repository define")
}

///
/// # Panics
///
fn description() -> String {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package
        .description
        .as_ref()
        .expect("missing description")
        .to_string()
}

fn authors() -> Vec<String> {
    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let package: &cargo_metadata::Package = metadata.packages.first().unwrap();
    package.authors.clone()
}
fn clear() {
    if OS.eq("windows") {
        assert!(Command::new("cls")
            .current_dir(".")
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success());
    } else {
        assert!(Command::new("clear")
            .current_dir(".")
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success());
    }
}

fn commit_types_with_help() -> [&'static str; 68] {
    let mut x = COMMITS_TYPES;
    x.sort_unstable();
    x
}

fn commit_scope() -> String {
    let mut scope: String;
    loop {
        scope = Text::new("Please enter the commit scope : ")
            .prompt()
            .unwrap();
        if scope.is_empty() {
            continue;
        }
        if scope.len().gt(&20) {
            println!("scope can be superior to 20 character");
            continue;
        }
        if confirm(
            format!("Really use the commit scope : {scope}").as_str(),
            false,
        ) {
            break;
        }
    }
    scope
}

fn get_commit_types() -> String {
    let mut t: String;
    loop {
        t = Select::new(
            "Please enter the commit type : ",
            commit_types_with_help().to_vec(),
        )
        .prompt()
        .unwrap()
        .to_string();
        if t.is_empty() {
            continue;
        }
        if confirm(format!("Really use the commit type : {t}").as_str(), false) {
            break;
        }
    }
    let x: Vec<&str> = t.split(':').collect();
    let mut s: String = String::from("\n");
    s.push_str((*x.first().unwrap()).to_string().as_str());
    s
}

fn commit_summary() -> String {
    let mut summary: String;
    loop {
        summary = Text::new("Please enter the commit summary : ")
            .prompt()
            .unwrap();
        if summary.is_empty() {
            continue;
        }
        if summary.len().gt(&50) {
            println!("Summary must be contains less than 50 chararacter");
            continue;
        }
        if confirm(format!("Use the summary : {summary}").as_str(), false) {
            break;
        }
    }
    summary
}

fn commit_why() -> String {
    let mut why: String = String::new();
    loop {
        let w = Text::new("Please explain the reasoning behind the change : ")
            .prompt()
            .unwrap();
        if w.is_empty() {
            continue;
        }
        if w.len().gt(&50) {
            println!("The reasoning behind the change must be contains less than 50 chararacter");
            continue;
        }
        why.push_str(format!("\n\t\t* {w}").as_str());
        if confirm("Continue to write the changes : ", false) {
            continue;
        }
        break;
    }
    why
}
fn commit_footer() -> String {
    let mut footer: String = String::new();
    if confirm("Code has breaking changes ?", false) {
        footer.push_str("BREAKING CHANGE: ");
        loop {
            let b = Text::new("Please enter the breaking change description: ")
                .prompt()
                .unwrap();
            if b.is_empty() {
                continue;
            }
            if confirm(
                format!("Use breaking change description : {b}").as_str(),
                false,
            ) {
                footer.push_str(b.as_str());
                break;
            }
        }
    }
    if confirm("Code has resolving issues ?", false) {
        loop {
            footer.push_str("\n\tFixes ");
            let f = Text::new("Please enter the issue number : ")
                .prompt()
                .unwrap();
            if f.is_empty() {
                continue;
            }
            footer.push_str(format!("#{f}\n").as_str());
            if confirm("Code resolving an other issues ?", false) {
                continue;
            }
            break;
        }
    }
    if confirm("Code resolve an issue ?", false) {
        loop {
            footer.push_str("\n\tCloses ");
            let f = Text::new("Please enter the issue number : ")
                .prompt()
                .unwrap();
            if f.is_empty() {
                continue;
            }
            footer.push_str(format!("#{f}\n").as_str());
            if confirm("Code resolve an other issue ?", false) {
                continue;
            }
            break;
        }
    }
    footer
}

fn get_scope() -> String {
    let mut scope: String;
    loop {
        scope = commit_scope();
        if check_commit(scope.as_str()) {
            break;
        }
    }
    scope
}

fn get_summary() -> String {
    let mut summary: String;
    loop {
        summary = commit_summary();
        if check_commit(summary.as_str()) {
            break;
        }
    }
    summary
}

fn get_why() -> String {
    let mut why: String;
    loop {
        why = commit_why();
        if check_commit(why.as_str()) {
            break;
        }
    }
    why
}
fn get_footer() -> String {
    let mut footer: String;
    loop {
        footer = commit_footer();
        if check_commit(footer.as_str()) {
            break;
        }
    }
    footer
}

fn confirm(msg: &str, default: bool) -> bool {
    Confirm::new(msg)
        .with_default(default)
        .prompt()
        .unwrap()
        .eq(&true)
}

fn email() -> String {
    String::from_utf8(
        Command::new("git")
            .arg("config")
            .arg("--get")
            .arg("user.email")
            .current_dir(".")
            .output()
            .expect("git email not found")
            .stdout,
    )
    .expect("msg")
    .trim()
    .to_string()
}

fn name() -> String {
    String::from_utf8(
        Command::new("git")
            .arg("config")
            .arg("--get")
            .arg("user.name")
            .current_dir(".")
            .output()
            .expect("username not found")
            .stdout,
    )
    .expect("msg")
    .trim()
    .to_string()
}

fn remove_branch(b: &str, r: &str) -> bool {
    Command::new("git")
        .arg("branch")
        .arg("-d")
        .arg(b)
        .current_dir(r)
        .spawn()
        .expect("git")
        .wait()
        .unwrap()
        .success()
}
fn remove_tag(t: &str, r: &str) -> bool {
    Command::new("git")
        .arg("tag")
        .arg("-d")
        .arg(t)
        .current_dir(r)
        .spawn()
        .expect("git")
        .wait()
        .unwrap()
        .success()
}

fn send(path: &str) -> bool {
    let repo: Repository = open(path);
    let mut remote_names: Vec<String> = Vec::new();

    // Iterate over all configured remotes
    for remote_name in &repo.remotes().expect("No remote has been founded") {
        let remote_name = remote_name.unwrap_or_default(); // Handle optional remote names
        remote_names.push(String::from(remote_name));
    }

    for remote_name in &remote_names {
        let mut remote = repo.find_remote(remote_name).expect("Failed to get remote");

        // Update remote refs to get latest changes
        remote
            .fetch(&["HEAD"], None, None)
            .expect("Failed to fetch");
        remote
            .push(&["refs/heads/*:refs/heads/*"], None)
            .expect("msg");
    }
    true
}
fn show_status(path: &str) {
    let repo: Repository = open(path);
    let mut opts: StatusOptions = StatusOptions::new();

    let statuses = repo
        .statuses(Some(
            opts.include_ignored(false)
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ))
        .expect("Failed to get status");

    for entry in &statuses {
        let path = entry.path().unwrap_or("unknown");
        let status = entry.status();
        println!("{path}: {status:?}");
    }
}
fn options() -> Vec<String> {
    let mut options: Vec<String> = Vec::new();
    for option in OPTIONS {
        options.push(option.to_string());
    }
    options
}

fn menu() -> Option<String> {
    let mut o: Vec<String> = options();
    o.sort_unstable();
    Select::new("Select an option below : ", o)
        .prompt_skippable()
        .expect("msg")
}

fn repositories() -> Vec<String> {
    let mut repositories: Vec<String> = Vec::new();
    let starting_directory: String =
        std::env::var(CRATES_PATH).expect("missing CRATES_PATH env variable");
    for entry in WalkDir::new(starting_directory.as_str()) {
        let entry: walkdir::DirEntry = entry.expect(""); // Handle potential errors
        if entry.file_type().is_dir()
            && entry
                .path()
                .parent()
                .expect("no parent")
                .eq(Path::new(&starting_directory))
        {
            repositories.push(entry.path().to_str().expect("msg").to_string());
        }
    }
    repositories
}
fn clone() -> bool {
    let mut url: String;
    loop {
        url = Text::new("Please enter the repository url : ")
            .prompt()
            .unwrap();
        if url.is_empty() {
            continue;
        }
        break;
    }
    Command::new("git")
        .arg("clone")
        .arg("--quiet")
        .arg(url)
        .current_dir(std::env::var(CRATES_PATH).expect(CRATES_PATH).as_str())
        .spawn()
        .expect("missing info")
        .wait()
        .unwrap()
        .success()
}

fn gist() -> bool {
    let mut url: String;
    loop {
        url = Text::new("Please enter the gist url : ").prompt().unwrap();
        if url.is_empty() {
            continue;
        }
        break;
    }
    Command::new("git")
        .arg("clone")
        .arg("--quiet")
        .arg(url)
        .current_dir(std::env::var(CRATES_PATH).expect(CRATES_PATH).as_str())
        .spawn()
        .expect("missing info")
        .wait()
        .unwrap()
        .success()
}

fn repo() -> String {
    Select::new("Select a repository to manage", repositories())
        .prompt()
        .unwrap()
}

fn display_branches() -> bool {
    for branch in &branches(repo().as_str()) {
        println!("{branch}");
    }
    true
}
fn display_status() -> bool {
    show_status(repo().as_str());
    true
}

fn remove_branches() -> bool {
    let repo: String = repo();
    let branches: Vec<String> =
        MultiSelect::new("Select branch to remove", branches(repo.as_str()))
            .prompt()
            .unwrap();
    for branch in &branches {
        assert!(remove_branch(branch, repo.as_str()));
    }
    true
}
fn remove_tags() -> bool {
    let repo: String = repo();
    let tags: Vec<String> = MultiSelect::new("Select tags to remove", tags(repo.as_str()))
        .prompt()
        .unwrap();
    for tag in &tags {
        assert!(remove_tag(tag, repo.as_str()));
    }
    true
}
fn flow(z: bool) -> ExitCode {
    if z.eq(&false) {
        if confirm("Your code contains errors, do you want recheck it ?", true).eq(&true) {
            return flow(zuu());
        }
        return ExitCode::FAILURE;
    }
    let x: Option<String> = menu();
    if x.is_none() {
        return ExitCode::SUCCESS;
    }
    let todo: String = x.unwrap();
    match todo.as_str() {
        COMMIT => {
            assert!(commit(repo().as_str()));
        }
        CLONE_REPO => {
            assert!(clone());
        }
        SHOW_LOGS => {
            assert!(logs(repo().as_str()));
        }
        SHOW_DIFF => {
            assert!(diff(repo().as_str()));
        }
        CLONE_GIST => {
            assert!(gist());
        }
        SHOW_STATUS => {
            assert!(display_status());
        }
        SEND_TO_REMOTE => {
            assert!(send(repo().as_str()));
        }
        SHOW_BRANCHES => {
            assert!(display_branches());
        }
        REMOVE_BRANCHES => {
            assert!(remove_branches());
        }
        REMOVE_RELEASE => {
            assert!(remove_tags());
        }
        GENERATE_CHANGE_LOG => {
            assert!(create_changelog());
        }
        _ => {
            unreachable!();
        }
    }
    ExitCode::SUCCESS
}

fn tags(path: &str) -> Vec<String> {
    let repo: Repository = open(path);
    let mut tags: Vec<String> = Vec::new();
    let tag_names = repo.tag_names(None).expect("NO tags");

    for tag_name in &tag_names {
        let tag_name = tag_name.unwrap_or("<unnamed>");
        tags.push(tag_name.to_string());
    }
    tags
}

fn branches(path: &str) -> Vec<String> {
    let repo: Repository = open(path);
    let mut branches: Vec<String> = Vec::new();
    let all_branches: Branches<'_> = repo.branches(Some(BranchType::Local)).expect("msg");
    for branch in all_branches {
        let (branch, _) = branch.expect("msg");
        let branch_name = branch
            .name()
            .expect("Failed to get branch name")
            .unwrap_or("<unnamed>");
        branches.push(branch_name.to_string());
    }
    branches
}

fn open(path: &str) -> Repository {
    Repository::open(path).expect("Not a git repository")
}

fn logs(path: &str) -> bool {
    let repo: Repository = open(path);
    let mut revwalk: Revwalk<'_> = repo.revwalk().expect("msg"); // Create a Revwalk object to iterate through commits
    revwalk.push_head().expect("msg"); // Start from the HEAD commit

    for (_i, oid) in revwalk.enumerate().take(5) {
        let commit: Commit<'_> = repo.find_commit(oid.expect("msg")).expect("msg");
        let message = commit.message().unwrap_or("No commit message");
        println!("\n{message}\n");
    }
    true
}
fn main() -> ExitCode {
    flow(zuu())
}
