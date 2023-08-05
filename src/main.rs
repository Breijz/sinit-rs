use std::env;
use std::path::Path;
use std::process::{Command, exit};
use std::mem::MaybeUninit;
use libc;
use libc::{getpid, sigfillset, sigprocmask, alarm, sigwait, waitpid, c_int, sigset_t, SIG_BLOCK, WNOHANG, SIGUSR1, SIGCHLD, SIGALRM, SIGINT};

const DEF_ALARMTIME: u32 = 30;

#[derive(Clone, Copy)]
struct Signal {
    sig: c_int,
    handler: fn(),
}

static SIGMAP: [Signal; 4] = [
    Signal{sig: SIGUSR1, handler: sigpoweroff},
    Signal{sig: SIGCHLD, handler: sigreap},
    Signal{sig: SIGALRM, handler: sigreap},
    Signal{sig: SIGINT, handler: sigreboot}
];

mod config;

pub fn main() -> () {
    let mut sig: c_int = 0;

    unsafe {
        if getpid() != 1 {
            eprint!("Not running as PID 1");
            exit(1);
        }
    }
    env::set_current_dir(&Path::new("/")).unwrap();

    unsafe {
        let mut set: sigset_t = MaybeUninit::zeroed().assume_init();
        sigfillset(&mut set as *mut sigset_t);
        sigprocmask(SIG_BLOCK, &set, std::ptr::null_mut());

        Command::new(config::RCINITCMD);

        loop {
            alarm(DEF_ALARMTIME);
            sigwait(&mut set as *mut sigset_t, &mut sig as *mut c_int);

            for signal in SIGMAP.into_iter() {
                if signal.sig == sig {
                    (signal.handler)();
                    break;
                }
            }
        }
    }
}

fn sigpoweroff() {
    Command::new(config::RCPOWEROFFCMD);
}

fn sigreap() {
    unsafe {
        while waitpid(-1, std::ptr::null_mut(), WNOHANG) > 0 {}
        alarm(DEF_ALARMTIME);
    }
}

fn sigreboot() {
    Command::new(config::RCREBOOTCMD);
}
