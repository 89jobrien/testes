use testes_fuzz::fuzz;

fuzz! {
    fn seed_frames_do_not_panic {
        id: "frame-parser-seeds",
        subject: "frame parser",
        claim: "seed frames cannot panic the parser",
        target: "frame_parser",
        oracles: ["no-panic"],
        corpus: [b"", b"\x00\x01", b"abc"],
        body: |input: &[u8]| {
            let _ = input.len();
        }
    }
}

fuzz! {
    #[should_panic(expected = "did not pass")]
    fn seed_frame_that_panics_fails_the_test {
        id: "frame-parser-crash",
        subject: "frame parser",
        claim: "seed frames cannot panic the parser",
        target: "frame_parser",
        oracles: ["no-panic"],
        corpus: [b"ok", b"boom"],
        body: |input: &[u8]| {
            assert_ne!(input, b"boom", "parser panicked on seed input");
        }
    }
}

fuzz! {
    #[should_panic(expected = "Inconclusive")]
    fn empty_seed_corpus_is_not_a_pass {
        id: "frame-parser-empty",
        subject: "frame parser",
        claim: "seed frames cannot panic the parser",
        target: "frame_parser",
        oracles: [],
        corpus: [],
        body: |_input: &[u8]| {}
    }
}
