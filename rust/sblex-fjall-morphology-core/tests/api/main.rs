use exn::{Exn, ResultExt};
use sblex_fjall_morphology_core::FjallMorphology;
use temp_dir::TempDir;

#[derive(Debug)]
pub struct TestError;

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Test failed")
    }
}

impl std::error::Error for TestError {}

#[test]
fn build_and_load_morphology() -> Result<(), Exn<TestError>> {
    let make_error = || TestError;
    let tmp_dir = TempDir::with_prefix("test.db").or_raise(make_error)?;
    let mut morph = FjallMorphology::new(tmp_dir.path()).or_raise(make_error)?;

    morph
        .build_from_path("assets/testing/saldo.lex")
        .or_raise(make_error)?;

    let result = morph.lookup("dv").or_raise(make_error)?;
    assert!(result.is_none());

    let result = morph.lookup("dväljs").or_raise(make_error)?.unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&result).or_raise(make_error)?;
    insta::assert_json_snapshot!("lookup__dväljs", result_json);

    let result = morph.lookup_with_cont("dv").or_raise(make_error)?;
    let result_json: serde_json::Value = serde_json::from_slice(&result).or_raise(make_error)?;
    insta::assert_json_snapshot!("lookup_with_cont__dv", result_json);

    let result = morph.lookup_with_cont("dväljs").or_raise(make_error)?;
    let result_json: serde_json::Value = serde_json::from_slice(&result).or_raise(make_error)?;
    insta::assert_json_snapshot!("lookup_with_cont__dväljs", result_json);
    Ok(())
}
