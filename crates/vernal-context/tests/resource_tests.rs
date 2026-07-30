//! 资源模块测试 - 对标 Spring Resource 抽象

use std::io;
use vernal_context::resource::{
    ByteArrayResource, ClassPathResource, FileSystemResource, Resource, ResourceError,
};

// ============================================================================
// ByteArrayResource 测试
// ============================================================================

#[test]
fn test_byte_array_resource_exists() {
    let resource = ByteArrayResource::new(b"hello".to_vec());
    assert!(resource.exists());
}

#[test]
fn test_byte_array_resource_is_readable() {
    let resource = ByteArrayResource::new(b"hello".to_vec());
    assert!(resource.is_readable());
}

#[test]
fn test_byte_array_resource_filename_is_none() {
    let resource = ByteArrayResource::new(b"hello".to_vec());
    assert!(resource.filename().is_none());
}

#[test]
fn test_byte_array_resource_description_default() {
    let resource = ByteArrayResource::new(b"hello".to_vec());
    assert_eq!(resource.description(), "字节数组资源");
}

#[test]
fn test_byte_array_resource_description_custom() {
    let resource = ByteArrayResource::with_description(b"hello".to_vec(), "custom description");
    assert_eq!(resource.description(), "custom description");
}

#[test]
fn test_byte_array_resource_read_bytes() {
    let data = b"hello world";
    let resource = ByteArrayResource::new(data.to_vec());
    let result = resource.read_bytes().unwrap();
    assert_eq!(result, data);
}

#[test]
fn test_byte_array_resource_read_string() {
    let data = b"hello world";
    let resource = ByteArrayResource::new(data.to_vec());
    let result = resource.read_string().unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_byte_array_resource_read_string_invalid_utf8() {
    let data = vec![0xFF, 0xFE]; // Invalid UTF-8
    let resource = ByteArrayResource::new(data);
    let result = resource.read_string();
    assert!(result.is_err());
}

#[test]
fn test_byte_array_resource_empty() {
    let resource = ByteArrayResource::new(Vec::new());
    assert!(resource.exists());
    assert!(resource.is_readable());
    assert!(resource.read_bytes().unwrap().is_empty());
}

// ============================================================================
// ClassPathResource 测试
// ============================================================================

#[test]
fn test_class_path_resource_description() {
    let resource = ClassPathResource::new("test.txt");
    assert!(resource.description().contains("test.txt"));
}

#[test]
fn test_class_path_resource_filename() {
    let resource = ClassPathResource::new("test.txt");
    assert_eq!(resource.filename(), Some("test.txt"));
}

// ============================================================================
// FileSystemResource 测试
// ============================================================================

#[test]
fn test_file_system_resource_description() {
    let resource = FileSystemResource::new("/tmp/test.txt");
    assert!(resource.description().contains("/tmp/test.txt"));
}

#[test]
fn test_file_system_resource_filename() {
    let resource = FileSystemResource::new("/tmp/test.txt");
    assert_eq!(resource.filename(), Some("test.txt"));
}

#[test]
fn test_file_system_resource_exists_nonexistent() {
    let resource = FileSystemResource::new("/nonexistent/path/test.txt");
    assert!(!resource.exists());
}

#[test]
fn test_file_system_resource_is_readable_nonexistent() {
    let resource = FileSystemResource::new("/nonexistent/path/test.txt");
    assert!(!resource.is_readable());
}

// ============================================================================
// ResourceError 测试
// ============================================================================

#[test]
fn test_resource_error_not_found_display() {
    let error = ResourceError::NotFound("test.txt".to_string());
    assert_eq!(format!("{error}"), "资源未找到: test.txt");
}

#[test]
fn test_resource_error_not_readable_display() {
    let error = ResourceError::NotReadable("test.txt".to_string());
    assert_eq!(format!("{error}"), "资源不可读: test.txt");
}

#[test]
fn test_resource_error_io_display() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = ResourceError::Io(io_error);
    assert!(format!("{error}").contains("资源 IO 错误"));
}

#[test]
fn test_resource_error_from_io_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error: ResourceError = io_error.into();
    assert!(matches!(error, ResourceError::Io(_)));
}

#[test]
fn test_resource_error_is_std_error() {
    let error = ResourceError::NotFound("test.txt".to_string());
    let _: &dyn std::error::Error = &error;
}

#[test]
fn test_resource_error_debug() {
    let error = ResourceError::NotFound("test.txt".to_string());
    let debug = format!("{error:?}");
    assert!(debug.contains("NotFound"));
}
