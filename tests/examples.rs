use primer_lang::run_vm;

#[test]
fn square_root_example_runs() {
    let output = run_vm(include_str!("../examples/square_root.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "1.5\n",
            "1.41666666666666652\n",
            "1.41421568627450966\n",
            "1.41421356237468987\n",
            "1.41421356237309492\n",
        )
    );
}

#[test]
fn logistic_map_example_exposes_f32_and_f64_divergence() {
    let output = run_vm(include_str!("../examples/logistic_map.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "0.624000013\n",
            "0.62400000000000011\n",
            "0.915033638\n",
            "0.91503359999999978\n",
            "0.303213596\n",
            "0.30321373239705673\n",
            "0.82397294\n",
            "0.82397314304332092\n",
            "0.565661967\n",
            "0.56566147008786449\n",
            "0.958185136\n",
            "0.95818542824901176\n",
        )
    );
}

#[test]
fn boolean_comparisons_example_runs() {
    let output = run_vm(include_str!("../examples/boolean_comparisons.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "true\n", "false\n", "true\n", "true\n", "true\n", "true\n", "true\n", "true\n",
            "true\n",
        )
    );
}

#[test]
fn conditional_example_runs() {
    let output = run_vm(include_str!("../examples/conditional.prim")).unwrap();

    assert_eq!(output, "true\n42\n");
}

#[test]
fn while_square_root_example_runs() {
    let output = run_vm(include_str!("../examples/while_square_root.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "1.5\n",
            "1.41666666666666652\n",
            "1.41421568627450966\n",
            "1.41421356237468987\n",
            "1.41421356237309492\n",
        )
    );
}

#[test]
fn loop_control_example_runs() {
    let output = run_vm(include_str!("../examples/loop_control.prim")).unwrap();

    assert_eq!(output, "12\n6\n");
}

#[test]
fn for_sum_example_runs() {
    let output = run_vm(include_str!("../examples/for_sum.prim")).unwrap();

    assert_eq!(output, "14\n");
}

#[test]
fn euclidean_gcd_example_runs() {
    let output = run_vm(include_str!("../examples/euclidean_gcd.prim")).unwrap();

    assert_eq!(output, "21\n");
}

#[test]
fn fibonacci_example_runs() {
    let output = run_vm(include_str!("../examples/fibonacci.prim")).unwrap();

    assert_eq!(output, "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n");
}

#[test]
fn factorial_example_runs() {
    let output = run_vm(include_str!("../examples/factorial.prim")).unwrap();

    assert_eq!(output, "3628800\n");
}

#[test]
fn collatz_example_runs() {
    let output = run_vm(include_str!("../examples/collatz.prim")).unwrap();

    assert_eq!(output, "111\n9232\n");
}

#[test]
fn prime_check_example_runs() {
    let output = run_vm(include_str!("../examples/prime_check.prim")).unwrap();

    assert_eq!(output, "true\n");
}

#[test]
fn integer_square_root_example_runs() {
    let output = run_vm(include_str!("../examples/integer_square_root.prim")).unwrap();

    assert_eq!(output, "1414\n");
}

#[test]
fn exponentiation_by_squaring_example_runs() {
    let output = run_vm(include_str!("../examples/exponentiation_by_squaring.prim")).unwrap();

    assert_eq!(output, "1594323\n");
}

#[test]
fn pythagorean_triples_example_runs() {
    let output = run_vm(include_str!("../examples/pythagorean_triples.prim")).unwrap();

    assert_eq!(output, "3\n4\n5\n5\n12\n13\n6\n8\n10\n9\n12\n15\n");
}

#[test]
fn product_point_example_runs() {
    let output = run_vm(include_str!("../examples/product-point.prim")).unwrap();

    assert_eq!(output, "0\n2\n");
}

#[test]
fn functions_example_runs() {
    let output = run_vm(include_str!("../examples/functions.prim")).unwrap();

    assert_eq!(output, "42\n");
}

#[test]
fn fixed_arrays_example_runs() {
    let output = run_vm(include_str!("../examples/fixed_arrays.prim")).unwrap();

    assert_eq!(output, "108\n4\n");
}

#[test]
fn product_arrays_example_runs() {
    let output = run_vm(include_str!("../examples/product_arrays.prim")).unwrap();

    assert_eq!(output, "3\n6\n2\n2\n100\n");
}

#[test]
fn xor_neural_network_example_runs() {
    let output = run_vm(include_str!("../examples/xor_neural_network.prim")).unwrap();

    assert_eq!(
        output,
        concat!(
            "0\n", "0\n", "0\n", "true\n", "0\n", "1\n", "1\n", "true\n", "1\n", "0\n", "1\n",
            "true\n", "1\n", "1\n", "0\n", "true\n",
        )
    );
}
