# corepc project instructions

Task: we are building codegen module. A completely standalone codegen module from bitcoin core
openrpc specs that produces client+into_model conversions code for an async production client for corepc

Your primary task is to ensure @codegen works properly.

Before handing off any code to me, make sure that you run `just codegen clean` and `just codegen 31`
(we are testing version 31 only now). Make sure that integration tests for async tests are independent
of sync tests! The @integration_tests module is hybrid and runs for both independant surfaces!!
Meaning you can do to folder "integration_tests/" and run `cargo test --features=31_0,download,test-async`
and `cargo test --features=31_0,download` (sync client) successfully.

NOTE: To ensure we are not touching sync tests, I MAY or MAY NOT have renamed the `into.rs` conversions for NON-ASYNC CLIENT
to `into.rs.md` and you are not to rename these back!!! Furthermore, many files in `client_sync` may have also been disabled for good measure. however keep in mind that SYNC TESTS SHOULD ALSO PASS AND SYNC FUNCTIONALITY MUST BE WHOLY UNAFFECTED!

If you must test sync, then pls rename from `.md` back to pure `.rs`, test than rename it back! And make sure that, while sync is disabled, the async tests run e2e

In an ideal world, you would modify only the @codegen module to make it all work. I expect as little modifcations
as possible to integration_tests, to non-generate `types/` or `client/`.

DO NOT MODIFY THE SPEC OPENRPC FILES! DO NOT EVEN FORMAT THEM. THEY SHOULD BE AS-IS!

## Bug-hunting is a first-class goal

One of the goals of the codegen `into_model` work is to **catch bugs in the existing
hand-written code** (mostly `types/`), especially places where the canonical `crate::model`
types convert Core's JSON into the wrong Rust type.

When you find (or strongly suspect) such a bug:

1. Confirm it. Research online (Core RPC docs, Core source, functional tests, example values)
   before calling it a bug. Do not flag on a hunch.
2. Log it in `/home/renato/Desktop/rust-bitcoin/corepc_bugs_backlog.md` as a new numbered entry:
   where (file:line), what (wrong vs correct), evidence, status, and how codegen handles it.
3. Leave the affected portion of the generated code **half-done**, do not paper over it. In the
   generated `into_model`:
   - Leave the field that would feed the wrong canonical type unimplemented (a `todo!()` or an
     equivalent placeholder that does not compile a wrong conversion).
   - Add a comment with **what you think the correct type/conversion is**.
   - Note **what the (buggy) canonical type currently expects to receive**, so the mismatch is
     visible at the call site.
   - Finish with a juicy `TODO` comment so each case can be referenced and fixed later,
     one by one.

We leverage rust-bitcoin types wherever possible. But when `types/` itself is wrong, we generate
what *looks correct*, comment it, record the canonical type's (wrong) expectation, and leave the
`TODO` rather than silently reproducing the bug.
