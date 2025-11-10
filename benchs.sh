#/bin/bash

cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 1 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 1 --work-ns 10000

cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 1 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 1 --work-ns 10000

cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 1 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 1 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 1 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 1 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 1 --work-ns 10000

# capacity 10

cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 1 --capacity 10 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 1 --capacity 10 --work-ns 10000

cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 1 --consumers 4 --capacity 10 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 1 --consumers 4 --capacity 10 --work-ns 10000

cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 10 --work-ns 0
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 10 --work-ns 100
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 10 --work-ns 1000
cargo r --release -p pq-bench -- --implm xbeam  --producers 4 --consumers 4 --capacity 10 --work-ns 10000
cargo r --release -p pq-bench -- --implm syncpq --producers 4 --consumers 4 --capacity 10 --work-ns 10000
