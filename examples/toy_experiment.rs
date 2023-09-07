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


fn print_r1cs_matrix<Fr: PrimeField>(r1cs: &nova_scotia::circom::circuit::R1CS<Fr>) {
    if r1cs.constraints.is_empty() {
        panic!("r1cs.constraints is empty!");
    }

    // 最大のidxの値を取得
    let m = r1cs.constraints.iter().flat_map(|(a, b, c)| {
        a.iter().chain(b.iter()).chain(c.iter()).map(|(idx, _)| *idx)
    }).max().unwrap_or(0) + 1;

    let n = r1cs.constraints.len(); // 制約の数

    // 初期化
    let mut matrix_a: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];
    let mut matrix_b: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];
    let mut matrix_c: Vec<Vec<Fr>> = vec![vec![Fr::from(0); m]; n];

    // 行列の構築
    for (i, (a, b, c)) in r1cs.constraints.iter().enumerate() {
        for (idx, coeff) in a {
            matrix_a[i][*idx] = *coeff;
        }
        for (idx, coeff) in b {
            matrix_b[i][*idx] = *coeff;
        }
        for (idx, coeff) in c {
            matrix_c[i][*idx] = *coeff;
        }
    }

    // 行列の表示
    println!("Matrix A:");
    for row in &matrix_a {
        println!("{:?}", row);
    }

    println!("\nMatrix B:");
    for row in &matrix_b {
        println!("{:?}", row);
    }

    println!("\nMatrix C:");
    for row in &matrix_c {
        println!("{:?}", row);
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
    print_r1cs_matrix(&r1cs);
}

