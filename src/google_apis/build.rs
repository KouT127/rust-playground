fn main() {
    tonic_build::configure()
        .out_dir("src/")
        .compile(
            &[
                "proto/google/firestore/v1/firestore.proto",
                "proto/google/firestore/v1beta1/firestore.proto",
            ],
            &["proto"],
        )
        .expect("failed to compile protos");
}
