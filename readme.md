# Command

```
cd main
cargo run -- {path of input csv} {path of output csv}
```
The output will be generated to both csv and stdout.

# Package Structure

## main
- This is where command line interface is built in

## service
- This is where IO operation logic is built in
- There are some integration test to prove that input csv file is properly read

## domain
- This is where the domain logic is built in
- THere are some unit test to prove that domain logic is right
- There is no IO operation in this project

# TODO

- There is a case that avaliable amount become minus(testData1.csv).
```
deposit, 1, 1, 1.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1, 3,
chargeback, 1, 3,
```
In this case, tx1 is chargeback after withdrawal, so according to the specification, the available become -0.5.

- Using tokio runtime for non-blocking IO

At this moment, the program run like a single batch operation. So I thought that it is overkill to use tokio runtime.

However, if the code is bundled in a web server and should handle thousands of TCP stream, than I will use tokio runtime(async await feature and also channel)