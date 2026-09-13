use std::thread;
use std::time::Duration;

async fn do_stuff_async() {
    // async work
}

async fn more_async_work() {
    // more here
}

#[tokio::main]
async fn main() {
    tokio::select! {
    _ = do_stuff_async() => {
        println!("do_stuff_async() completed first")
    }
    _ = more_async_work() => {
        println!("more_async_work() completed first")
    }
    };

    let mut count = 0u8;

    loop {
        tokio::select! {
        biased;

        _ = async {}, if count < 1 => {
            count += 1;
            assert_eq!(count, 1);
        }
        _ = async {}, if count < 2 => {
            count += 1;
            assert_eq!(count, 2);
        }
        _ = async {}, if count < 3 => {
            count += 1;
            assert_eq!(count, 3);
        }
        _ = async {}, if count < 4 => {
            count += 1;
            assert_eq!(count, 4);
        }
        else => {
            break;

        }
    };

        thread::sleep(Duration::from_millis(50));
        tokio::time::sleep(Duration::from_millis(50)).await
    }


    use tokio::sync::oneshot;

    let (tx1, mut rx1) = oneshot::channel();

    let (tx2, mut rx2) = oneshot::channel();

    tokio::spawn(async move {
        tx1.send("first").unwrap();
        // loop {
        //     tokio::select! {
        //
        //     }
        // }
    });

    tokio::spawn(async move {
        tx2.send("second").unwrap();
    });

    let mut a = None;
    let mut b = None;

    while a.is_none() || b.is_none() {
        tokio::select! {
        v1 = (&mut rx1), if a.is_none() => a = Some(v1.unwrap()),
        v2 = (&mut rx2), if b.is_none() => b = Some(v2.unwrap()),
    }
    }

    let res = (a.unwrap(), b.unwrap());

    assert_eq!(res.0, "first");
    assert_eq!(res.1, "second");
}
