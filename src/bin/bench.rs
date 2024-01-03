use crossbeam::channel as cb_channel;
use mass_transit::channel;
use std::sync::mpsc::channel as std_channel;
use std::thread;
use std::time::Instant;

fn generate_data(id: u8) -> Vec<String> {
    (0..100_000).map(|item| format!("{id}-{item}")).collect()
    // (0..5).map(|item| format!("{id}-{item}")).collect()
}

fn main() {
    run_custom_one();
    run_custom_exact(1000);
    run_custom_all();
    run_std();
    run_crossbeam();
}

fn run_std() {
    let (tx, rx) = std_channel();
    let time = Instant::now();
    thread::scope(|scope| {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        scope.spawn(move || {
            for item in generate_data(1) {
                tx.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx2.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx3.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx4.send(item);
            }
        });
        scope.spawn(move || {
            let mut counter = 0;
            while let Ok(msg) = rx.recv() {
                counter += 1;
            }
        });
    });
    let took = time.elapsed();
    println!("Standard took {:?}", took);
}

fn run_crossbeam() {
    let (tx, rx) = cb_channel::unbounded();
    let time = Instant::now();
    thread::scope(|scope| {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        scope.spawn(move || {
            for item in generate_data(1) {
                tx.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx2.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx3.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx4.send(item);
            }
        });
        scope.spawn(move || {
            let mut counter = 0;
            while let Ok(msg) = rx.recv() {
                counter += 1;
            }
        });
    });
    let took = time.elapsed();
    println!("CrossBeam took {:?}", took);
}

fn run_custom_one() {
    let (tx, rx) = channel();
    let time = Instant::now();
    thread::scope(|scope| {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        scope.spawn(move || {
            for item in generate_data(1) {
                tx.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx2.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(3) {
                tx3.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(4) {
                tx4.send(item);
            }
        });
        scope.spawn(move || {
            let mut counter = 0;
            while let Some(msg) = rx.recv() {
                // dbg!(msg);
                counter += 1;
            }
        });
    });
    let took = time.elapsed();
    println!("Custom recv took {:?}", took);
}

fn run_custom_exact(num: usize) {
    let (tx, rx) = channel();
    let time = Instant::now();
    thread::scope(|scope| {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        scope.spawn(move || {
            for item in generate_data(1) {
                tx.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx2.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx3.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx4.send(item);
            }
        });
        scope.spawn(move || {
            let mut counter = 0;
            while let Some(msg) = rx.recv_exact(num) {
                counter += 1;
            }
        });
    });
    let took = time.elapsed();
    println!("Custom recv_exact {:?}", took);
}

fn run_custom_all() {
    let (tx, rx) = channel();
    let time = Instant::now();
    thread::scope(|scope| {
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        let tx4 = tx.clone();
        scope.spawn(move || {
            for item in generate_data(1) {
                tx.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx2.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx3.send(item);
            }
        });
        scope.spawn(move || {
            for item in generate_data(2) {
                tx4.send(item);
            }
        });
        scope.spawn(move || {
            let mut counter = 0;
            while let Some(msg) = rx.recv_all() {
                counter += 1;
            }
        });
    });
    let took = time.elapsed();
    println!("Custom recv_all {:?}", took);
}
