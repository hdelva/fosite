extern crate skeptic;
#[test] fn readme_0() {
    let ref s = format!("{}", r####"extern crate carboxyl;

fn main() {
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();
    let signal = stream.hold(3);

    // The current value of the signal is initially 3
    assert_eq!(signal.sample(), 3);

    // When we fire an event, the signal get updated accordingly
    sink.send(5);
    assert_eq!(signal.sample(), 5);
}
"####);
    skeptic::rt::run_test(r#"/home/harm/forsite/rust/fosite/target/debug/build/carboxyl-d875304b1cca4e2d/out"#, s);
}

#[test] fn readme_1() {
    let ref s = format!("{}", r####"extern crate carboxyl;

fn main() {
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();

    let mut events = stream.events();
    sink.send(4);
    assert_eq!(events.next(), Some(4));
}
"####);
    skeptic::rt::run_test(r#"/home/harm/forsite/rust/fosite/target/debug/build/carboxyl-d875304b1cca4e2d/out"#, s);
}

#[test] fn readme_2() {
    let ref s = format!("{}", r####"extern crate carboxyl;

fn main() {
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();

    let squares = stream.map(|x| x * x).hold(0);
    sink.send(4);
    assert_eq!(squares.sample(), 16);
}
"####);
    skeptic::rt::run_test(r#"/home/harm/forsite/rust/fosite/target/debug/build/carboxyl-d875304b1cca4e2d/out"#, s);
}

#[test] fn readme_3() {
    let ref s = format!("{}", r####"extern crate carboxyl;

fn main() {
    let sink = carboxyl::Sink::new();
    let stream = sink.stream();

    let negatives = stream.filter(|&x| x < 0).hold(0);

    // This won't arrive at the signal.
    sink.send(4);
    assert_eq!(negatives.sample(), 0);

    // But this will!
    sink.send(-3);
    assert_eq!(negatives.sample(), -3);
}
"####);
    skeptic::rt::run_test(r#"/home/harm/forsite/rust/fosite/target/debug/build/carboxyl-d875304b1cca4e2d/out"#, s);
}

