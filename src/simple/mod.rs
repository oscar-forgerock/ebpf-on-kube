mod oomkill {
    include!(concat!(env!("OUT_DIR"), "/oomkill.skel.rs"));
}

use libbpf_rs::RingBufferBuilder;
use oomkill::*;
use plain::Plain;
use procfs::process::Process;
use prometheus_client::{
    encoding::{EncodeLabelSet, EncodeLabelValue},
    metrics::{counter::Counter, family::Family},
    registry::Registry,
};
use std::sync::Arc;
use std::time::Duration;
unsafe impl Plain for oomkill_bss_types::event {}

pub struct Simple {
    counter: Family<OomKillLabels, Counter>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct OomKillLabels {
    env: Env,
    c_group: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
enum Env {
    Production,
    Staging,
}

#[derive(PartialEq, Debug)]
struct MetricData {
    c_group: String,
}

// this function does our mapping. We can't get everything from the kernel so
// while in userspace let's grab some info in procfs.
fn handle_oom_kill(data: &[u8]) -> MetricData {
    // this event is the same as our C version but the libbpf crates do some
    // helpful parsing for us which is nice
    let mut event = oomkill_bss_types::event::default();
    // we get a chunk of memory in the form of an array of bytes. We need to tell
    // our program that these bytes are actually our struct
    plain::copy_from_bytes(&mut event, data).expect("data buffer too short");
    // we shouldn't use unwrap in production but in the future we'll have nicer
    // error handling.
    // here we get another struct representing the procfs entry for the given pid
    // that was killed by the oom killer.
    let process_info = Process::new(event.pid).unwrap();
    // grab the commandline and do some ascii to utf8 hackery
    let raw_comm = event.my_comm.clone();
    let temp_comm = String::from_utf8_lossy(&raw_comm);
    let parsed_comm = temp_comm.trim_matches(char::from(0));
    // get the list of cgroups for the process
    let cgroups = process_info.cgroups().unwrap();

    // I'm not sure if a process can belong to multiple cgroups but we're looking
    // for the path that ends in .scope since that's what docker seems to do
    let cgroup_opt = cgroups
        .iter()
        .find(|&group| group.pathname.contains("scope"));

    // this is more nicer handling, since I prefer to not panic we just set the
    // output to a default value if one isn't present.
    let cgroup_name = match cgroup_opt {
        Some(group) => group.pathname.as_str(),
        None => "no cgroup name",
    };

    // print our output this might be converted to an event or a trace since the
    // cardinality would be pretty intense with the pids and cgroup_names
    println!(
        "{:9} {:<7} {:<7} {:<7} {:<20} {:<10} {:<20}",
        "time", event.pid, event.ppid, event.cgroup, 1, parsed_comm, cgroup_name
    );

    // upstream libbpf-rs says this function must return an integer. IDK why but
    // the upstream told the compiler which means we have no choice in the matter.
    MetricData {
        c_group: cgroup_name.to_string(),
    }
}

impl Simple {
    fn simple2(self) {
        // let's build our probe with default options. You can set global variables in
        // your probe file and then set the values via this builder.
        let skel_builder = OomkillSkelBuilder::default();

        // all ebpf programs have this sequency of create builder -> open builder ->
        // loadbuilder -> attach
        let open_skel = skel_builder.open().unwrap();

        let mut skel = open_skel.load().unwrap();
        skel.attach().unwrap();
        // map is the same name as in the bpf.c file
        // we need to connect to our ring buffer that is in kernel space. think of this
        // like a pipe.
        let mut builder = RingBufferBuilder::new();
        // declare our callback function
        let wrapper = |data: &[u8]| -> i32 {
            let metric = handle_oom_kill(data);
            self.counter
                .get_or_create(&OomKillLabels {
                    env: Env::Staging,
                    c_group: metric.c_group,
                })
                .inc();
            0
        };

        builder.add(skel.maps_mut().rb(), wrapper).unwrap();
        let ring_buffer = builder.build().unwrap();
        println!("beginning to poll our perf buffer");
        // print a nice header for us
        println!(
            "{:9} {:<7} {:<7} {:<7} {:<20} {:<10} {:<20}",
            "timestamp", "pid", "ppid", "cgroup", "highwater memory", "cmdline", "cgroup_name"
        );
        // just poll the buffer for events every 100ms this may not be a production
        // best practice. From there we just enter an infinite loop and process events.
        loop {
            ring_buffer.poll(Duration::from_millis(100)).unwrap();
        }
    }

    pub fn start(self) {
        self.simple2()
    }
}

// TODO: find idiomatic way of doing this
pub async fn new_simple(registry: Arc<tokio::sync::Mutex<Registry>>) -> Simple {
    let oomkill_counter = Family::<OomKillLabels, Counter>::default();
    registry.lock().await.register(
        "syscall",
        "number of syscalls made by cgroup",
        oomkill_counter.clone(),
    );
    Simple {
        counter: oomkill_counter,
    }
}

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;
