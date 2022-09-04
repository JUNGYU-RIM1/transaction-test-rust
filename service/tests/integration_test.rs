use std::path::PathBuf;
#[test]
fn test_data1_should_be_deserialized_and_serialized_properly() {
    let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    file_path.push("tests/resources/testData1.csv");

    let path_string = file_path.into_os_string().into_string().unwrap();
    let result = service::service::read_csv(path_string).unwrap();
    println!("{:?}", result.get_user_account(1));
    println!("{:?}", result.get_user_account(2));

    let mut w_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    w_file_path.push("tests/resources/testDataOutput1.csv");
    let w_path_string = w_file_path.into_os_string().into_string().unwrap();
    service::service::write_csv(w_path_string, &result).unwrap();
}

#[test]
fn test_data2_should_be_deserialized_and_serialized_properly() {
    let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    file_path.push("tests/resources/testData2.csv");

    let path_string = file_path.into_os_string().into_string().unwrap();
    let result = service::service::read_csv(path_string).unwrap();
    println!("{:?}", result.get_user_account(1));
    println!("{:?}", result.get_user_account(2));

    let mut w_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    w_file_path.push("tests/resources/testDataOutput2.csv");
    let w_path_string = w_file_path.into_os_string().into_string().unwrap();
    service::service::write_csv(w_path_string, &result).unwrap();
}
