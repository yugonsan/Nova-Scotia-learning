pragma circom 2.0.3;

// include "https://github.com/0xPARC/circom-secp256k1/blob/master/circuits/bigint.circom";

template Example () {
    // a list of public inputs
    // (these must be named step_in for the Nova-Scotia interface) 
    signal input step_in;

    // outputs the same number of public outputs (named step_out).
    // These public outputs will then be routed to the next step of recursion as step_in,
    // and this will continue until we reach the end of the recursion iterations
    signal output step_out;

    // Circom circuits can input additional private inputs (with any name/JSON structure Circom will accept). 
    // We will instrument the piping of these private inputs in our Rust shimming.
    signal input adder;

    step_out <== step_in * adder;
}

component main { public [step_in] } = Example();

/* INPUT = {
    "step_in": [1, 1],
    "step_out": [1, 2],
    "adder": 0
} */