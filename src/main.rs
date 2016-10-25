extern crate clap;
extern crate libc;
extern crate nix;
extern crate tempdir;
extern crate walkdir;

use std::env;
use std::fmt;
use std::fs::{self, File};
use std::os::unix::prelude::*;
use std::path::Path;
use std::process::Command;

use clap::{App, Arg};
use clap::AppSettings::{ArgRequiredElseHelp, TrailingVarArg, UnifiedHelpMessage};

use nix::mount::{mount, umount2, MsFlags, MNT_DETACH, MS_BIND, MS_NOSUID, MS_REC, MS_PRIVATE};
use nix::sched::{unshare, CLONE_NEWUSER, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWUTS, CLONE_NEWNET,
                 CLONE_NEWIPC};
use nix::unistd::{getuid, getgid, pivot_root};

use tempdir::TempDir;

use walkdir::{WalkDir, WalkDirIterator};

macro_rules! check {
    ($e:expr) => (
        $e.unwrap_or_else(|e| panic!(concat!(stringify!($e), ": {:?}"), e))
    );
    ($e:expr, $msg:expr) => (
        $e.unwrap_or_else(|e| panic!(concat!($msg, ": {:?}"), e))
    )
}

// use oci? (ideally for forward compatibility)
// initially just pass a rootfs dir, no mounts supported
// type to rep aci manifest

// focus on just setting up the filesystem for now

type Error = Box<std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

const NONE: Option<&'static [u8]> = None;

fn main() {
    let mount_value_names = ["host-dir", "container-dir"];
    let matches = App::new("containy-thing")
                      .settings(&[ArgRequiredElseHelp, TrailingVarArg, UnifiedHelpMessage])
                      .arg(Arg::with_name("ROOTFS")
                               .help("Path to the extracted rootfs")
                               .required(true)
                               .index(1))
                      .arg(Arg::with_name("COMMAND").help("Command to run").index(2))
                      .arg(Arg::with_name("ARG")
                               .help("Arguments for COMMAND")
                               .multiple(true)
                               .requires("COMMAND")
                               .index(3))
                      .arg(Arg::with_name("mount")
                               .short("m")
                               .long("mount")
                               .help("Mount <host-dir> at <container-dir>")
                               .takes_value(true)
                               .value_names(&mount_value_names))
                      .get_matches();
    let rootfs = matches.value_of("ROOTFS").expect("ROOTFS should always be present");
    let command = matches.value_of("COMMAND").unwrap_or("/bin/bash");

    let (uid, gid) = (getuid(), getgid());

    check!(unshare(CLONE_NEWUSER | CLONE_NEWNS));

    // Figure out exactly how shared subtrees work and give this a meaningful comment!
    check!(mount(NONE, b"/".as_ref(), NONE, MS_REC | MS_PRIVATE, NONE));


    check!(mount(Some(Path::new(rootfs)),
                 Path::new(rootfs),
                 Some(Path::new("")),
                 MS_BIND | MS_NOSUID,
                 Some(Path::new(""))));
    // chdir("/jail");
    // unshare(CLONE_NEWNS);
    // mount("/jail", "/jail", NULL, MS_BIND, NULL);
    // pivot_root("/jail", "/jail/old_root");
    // chdir("/");
    // mount("/old_root/bin", "bin", NULL, MS_BIND, NULL);
    // mount("/old_root/usr", "usr", NULL, MS_BIND, NULL);
    // mount("/old_root/lib", "lib", NULL, MS_BIND, NULL);
    // umount2("/old_root", MNT_DETACH);
    // exec("/busybox");

    check!(env::set_current_dir(rootfs));
    // fs::create_dir(Path::new(rootfs).join("bin")).expect("create /bin");
    // fs::create_dir(Path::new(rootfs).join("lib")).expect("create /bin");
    // fs::create_dir(Path::new(rootfs).join("lib64")).expect("create /bin");

    // check!(mount(Some(Path::new("/bin")),
    //             &Path::new(rootfs).join("bin"),
    //             Some(Path::new("")),
    //             MS_BIND | MS_NOSUID,
    //             Some(Path::new(""))));

    // check!(mount(Some(Path::new("/lib")),
    //             &Path::new(rootfs).join("lib"),
    //             Some(Path::new("")),
    //             MS_BIND | MS_NOSUID,
    //             Some(Path::new(""))));

    // check!(mount(Some(Path::new("/lib64")),
    //             &Path::new(rootfs).join("lib64"),
    //             Some(Path::new("")),
    //             MS_BIND | MS_NOSUID,
    //             Some(Path::new(""))));

    // for entry in WalkDir::new(rootfs).into_iter().filter_map(|e| e.ok()) {
    //    println!("{}", entry.path().display());
    // }

    check!(setup_container_rootfs());


    // for entry in WalkDir::new(check!(env::current_dir())).into_iter().filter_entry(|e| !e.path().starts_with("old-root")).filter_map(|e| e.ok()) {
    //    println!("{}", entry.path().display());
    // }

    //    println!("/bin/bash: {:?}", Path::new("/bin/bash").exists());
    //    println!("/bin/bash: {:o}", Path::new("/bin/bash").metadata().map(|md| md.permissions().mode());

    check!(Command::new(command)
        .env_clear()
        .status(), "run command");

    // child process will be in new pid namespace

    // need to read in some source thing aci manifest
    // assume an ACI image layout (untarred)
    // set up the mounts
    // fork & exec
    // wait

}

// fn bind<S: AsRef<OsStr>, D: AsRef<OsStr>>(src: S, dst: D) -> Result<()> {
// }

/// Expects to have chdir'ed to the rootfs directory already.
fn setup_container_rootfs() -> Result<()> {
    // need to set up the root dir and chdir to it
    let old_root = try!(TempDir::new_in(check!(env::current_dir()), "old-root"));

    println!("old_root: {:?}", old_root.path());
    try!(pivot_root(b".".as_ref(), old_root.path()));

    let old_root = Path::new("/").join(old_root.into_path()
                                               .iter()
                                               .last()
                                               .expect("old_root should not be empty"));

    check!(env::set_current_dir("/"));

    check!(umount2(&old_root, MNT_DETACH));

    check!(fs::remove_dir(old_root));

    Ok(())
    // fs::create_dir("old-root").
}
