Day 8 Dataset Files
===================

day08example.txt     - The example dataset (20 coordinates) from the problem statement
                       Expected result: 40 (5 × 4 × 2) after 10 connections
                       Used by the regression test in src/days/day08.rs

day08coordinates.txt - Full puzzle input (1000 coordinates)
                       Expected result: 67488 (57 × 37 × 32) after 1000 connections
                       Used by the main solution (cargo run 8)

Running Tests
=============

To run the regression test on the example (20 coordinates → 40):
    cargo test day08::tests::test_example -- --nocapture

To run the regression test on the full puzzle (1000 coordinates → 67488):
    cargo test day08::tests::test_full_puzzle -- --nocapture

To run all day 8 tests:
    cargo test day08::tests -- --nocapture

To run the full solution:
    cargo run --release 8



