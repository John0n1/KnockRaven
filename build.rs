fn main() {
    // Build the C sequence generation library using the cc crate.  The library
    // consists of a simple recursive generator that computes all possible
    // sequences of a given length from a set of ports.  By compiling it in the
    // build script we avoid shipping precompiled binaries and ensure
    // portability across Linux targets.  See c_engine/seqgen.c for details.
    cc::Build::new()
        .file("c_engine/seqgen.c")
        .include("c_engine")
        .compile("seqgen");
}