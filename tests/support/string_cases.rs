// 全経路に同じ入力と既知の期待バイト列を渡します。
pub const CASES: &[(&str, &str)] = &[
    (
        r#"
        print(""); print("日本語\0\r\n\t\"\\\u{1f600}");
        print("a\0x" != "a\0y"); print("a\0" == "a"); print("" == "");
        print("\u{e9}" == "e\u{301}"); print("日本語" == "\u{65e5}本語");
        print(42); print(true); print(1.5f64); print("end");
    "#,
        "\n日本語\0\r\n\t\"\\😀\ntrue\nfalse\ntrue\nfalse\ntrue\n42\ntrue\n1.5\nend\n",
    ),
    (
        r#"
        type Label { id: i64, text: string = "既定", }
        fn identity(text: string) -> string { return text; }
        fn forward(text: string) -> string { return identity(text); }
        fn make() -> [[Label; 1]; 2] {
            mut rows: [[Label; 1]; 2] = [[Label { id: 0, }], [Label { id: 1, text: "保存", }]];
            saved: infer = rows;
            rows[1][0] = Label { id: 2, text: "変更", };
            print(rows[1][0].text);
            return saved;
        }
        fn replace(original: [string; 2]) -> [string; 2] {
            mut words: infer = original;
            words[0] = "replacement";
            return words;
        }
        fn main() -> void {
            original: infer = make();
            mut copy: infer = original;
            for (mut i: i64 = 0; i < 3; i = i + 1) {
                copy[1][0] = Label { id: 3, text: forward("\0end"), };
            }
            print(original[0][0].text); print(original[1][0].text); print(copy[1][0].text);
            mut text: string = "old"; saved: infer = text; text = "new";
            print(saved); print(text);
            words: [string; 2] = ["first", "second"];
            changed: infer = replace(words);
            print(words[0]); print(changed[0]); print(changed[1]);
        }
    "#,
        "変更\n既定\n保存\n\0end\nold\nnew\nfirst\nreplacement\nsecond\n",
    ),
    (
        r#"
        fn mark(text: string) -> string { print(text); return text; }
        fn pair(left: string, right: string) -> string { return right; }
        fn index() -> i64 { print("index"); return 0; }
        fn fail() -> string { print(["bad"][1]); return "bad"; }
        type Pair { first: string = mark("default"), second: string, }
        print(mark("left") == mark("right"));
        print(pair(mark("arg1"), mark("arg2")));
        value: Pair = Pair { second: mark("explicit"), }; print(value.first);
        mut words: [string; 2] = [mark("item1"), mark("item2")];
        words[index()] = mark("replacement"); print(words[0]);
        print(false && fail() == "bad"); print(true || fail() != "bad");
        for (mut flag: bool = true; flag; flag = mark("update1") == mark("update2")) {
            if mark("condition1") != mark("condition2") { continue; }
        }
    "#,
        "left\nright\nfalse\narg1\narg2\narg2\nexplicit\ndefault\ndefault\nitem1\nitem2\nindex\nreplacement\nreplacement\nfalse\ntrue\ncondition1\ncondition2\nupdate1\nupdate2\n",
    ),
];

pub const UNUSED_DEFAULT: &str =
    r#"type Unused { flag: bool = "a" == "a", } print(1); print(true);"#;

pub const BYTE_LENGTH: (&str, &str) = (
    include_str!("../../examples/string_byte_length.prim"),
    "0\n9\n3\n2\n3\n4\n7\n3\n9\nleft\nright\n9\nfalse\nfalse\n6\n10\n",
);

pub const OUT_OF_BOUNDS: &[&str] = &[
    r#"print(byte_len(["a"][1]));"#,
    r#"print(["a"][-1]);"#,
    r#"mut values: [string; 1] = ["a"]; values[1] = "replacement";"#,
];
