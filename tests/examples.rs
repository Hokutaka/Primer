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
