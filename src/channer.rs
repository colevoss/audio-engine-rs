use crossbeam::channel;
use std::{thread, time::Duration};

fn test() {
    channel_threads_test()
}

struct Channel {}

impl Channel {
    pub fn test(&self) {
        println!("CHANNEL TEST");
    }
}

fn try_with_channels() {
    let channels = vec![Channel {}, Channel {}, Channel {}, Channel {}];

    for chan in channels {
        let t = thread::spawn(move || {
            chan.test();
            println!("asdcf")
        });
    }
}

fn channel_threads_test() {
    let channels = vec![
        vec![101, 102, 103, 104, 105, 106, 107, 108, 109, 110],
        vec![201, 202, 203, 204, 205, 206, 207, 208, 209, 210],
        vec![301, 302, 303, 304, 305, 306, 307, 308, 309, 310],
        vec![401, 402, 403, 404, 405, 406, 407, 408, 409, 410],
        vec![501, 502, 503, 504, 505, 506, 507, 508, 509, 510],
        vec![601, 602, 603, 604, 605, 606, 607, 608, 609, 610],
        vec![701, 702, 703, 704, 705, 706, 707, 708, 709, 710],
        vec![801, 802, 803, 804, 805, 806, 807, 808, 809, 810],
        vec![901, 902, 903, 904, 905, 906, 907, 908, 909, 910],
        vec![1001, 1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010],
    ];

    // let (tx, rx) = channel::bounded::<i32>(channels.len());
    let mut chan_receivers = Vec::with_capacity(channels.len());

    let mut handlers = Vec::new();
    for chan in channels {
        let (tx, rx) = channel::bounded(chan.len() / 2);
        chan_receivers.push(rx);
        // When is this thread done? When iter => None?
        let t = thread::spawn(move || {
            let mut iter = chan.iter();
            while let Some(sample) = iter.next() {
                // println!("Got Sample {}", { sample });
                thread::sleep(Duration::from_secs(*sample / 100));
                // println!("Sending sample {}", { sample });
                match tx.send(*sample) {
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                    _ => {}
                }
            }

            loop {
                tx.send(0);
            }
        });

        handlers.push(t);
    }

    let (mixer_tx, mixer_rx) = channel::bounded(10);

    let adder = thread::spawn(move || {
        println!("HI");
        loop {
            let mut i = 0;
            // while let Some(rec) = chan_receivers.iter().next() {
            //     match rec.recv() {
            //         Ok(samp) => {
            //             println!("SAMP: {}", samp);
            //             i += samp;
            //         }
            //         Err(e) => {
            //             println!("Removing channel");
            //             // drop(rec);
            //             // chan_receivers.retain(|r| !r.same_channel(rec));
            //         }
            //     }
            // }
            for rec in chan_receivers.iter_mut() {
                match rec.recv() {
                    Ok(samp) => {
                        println!("SAMP: {}", samp);
                        i += samp;
                    }
                    Err(e) => {
                        println!("Removing channel");
                        // drop(rec);
                        // chan_receivers.retain(|r| !r.same_channel(rec));
                    }
                }
            }

            // println!("I Total: {}", i);
            mixer_tx.send(i);
            // thread::sleep(Duration::from_millis(500));
            i = 0;
        }
    });

    while let Ok(mixed_sample) = mixer_rx.recv() {
        println!("Mixed: {}", mixed_sample);
    }

    for t in handlers {
        t.join().unwrap();
    }

    adder.join().unwrap();

    println!("Hello, world!");
}
