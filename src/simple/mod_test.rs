use super::*;

//use plain::Plain;
use std::mem;


#[test]
fn test_handle_oom_kill()
{

    // create an event object to pass into the handle_oom_kill
    let mut event = oomkill_bss_types::event::default();

    // configure any variables you neet to set
    event.pid = 10;

    // pass event struct to a byte array pointer
    let data: &[u8];

    unsafe {
        data = plain::as_bytes(&event); 
    }

    // assert anything you need to assert
    // bit dodge at the moment referencing pid 10
    assert_eq!(handle_oom_kill(data), 0);
}