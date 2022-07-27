/*
    let (tx, mut rx) = mpsc::channel(64);
    let req_read = tokio::spawn(async move {
    let (resp_tx, resp_rx) = oneshot::channel();

    // send a disk request to read
    tx.send(DiskRequest::Read {
            block_id: 0,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("read res = {:?}", res.unwrap());

        // WRITE REQ
        let (resp_tx, resp_rx) = oneshot::channel();

        let new_block = [1; 4096];

        // send a write req through tx..? why doesnt it work
        tx.send(DiskRequest::Write {
            block_id: 0,
            block: new_block,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("write res = {:?}", res.unwrap());

        let (resp_tx, resp_rx) = oneshot::channel();

        // send a disk request to read
        tx.send(DiskRequest::Read {
            block_id: 0,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("read res = {:?}", res.unwrap());
    });

    req_read.await.unwrap();
*/