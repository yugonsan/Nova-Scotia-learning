use std::{collections::HashMap, env::current_dir, time::Instant};

use ff::PrimeField;
use nova_scotia::{
    circom::reader::load_r1cs, create_public_params, create_recursive_circuit, FileLocation, F, S,
};

fn print_r1cs<Fr: PrimeField>(r1cs: &nova_scotia::circom::circuit::R1CS<Fr>) {
    println!("num_inputs: {}", r1cs.num_inputs);
    println!("num_aux: {}", r1cs.num_aux);
    println!("num_variables: {}", r1cs.num_variables);

    println!("constraints:");
    for (a, b, c) in &r1cs.constraints {
        print!("A:");
        for (idx, coeff) in a {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();
        
        print!("B:");
        for (idx, coeff) in b {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();

        print!("C:");
        for (idx, coeff) in c {
            print!(" (idx: {}, coeff: {:?})", idx, coeff); 
        }
        println!();
    }
}


fn main() {
    let group_name = "bn254";
    let circuit_filepath = format!("examples/toy/{}/toy.r1cs", group_name);

    let root = std::env::current_dir().unwrap();
    // The cycle of curves we use, can be any cycle supported by Nova
    type G1 = pasta_curves::pallas::Point;
    type G2 = pasta_curves::vesta::Point;

    let circuit_file = root.join(circuit_filepath);

    let r1cs = load_r1cs::<G1, G2>(&FileLocation::PathBuf(circuit_file));

    print_r1cs(&r1cs);
}

