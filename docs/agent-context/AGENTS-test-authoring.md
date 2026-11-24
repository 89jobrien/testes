# Test authoring

Read this before writing or changing tests, or before changing behavior that tests cover.

- For any changed behavior: unit coverage first, then add property/fuzz/model-
  check/conformance/integration/regression coverage only as relevant — don't treat a
  checklist of test-attribute names as a substitute for executed evidence.
- Prefer fakes over elaborate mocks when a direct value suffices. Give fallible test
  setup a contextual `expect(...)`, not a bare `unwrap()`.
