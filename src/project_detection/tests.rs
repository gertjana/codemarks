use super::*;
use std::path::Path;
use tempfile::TempDir;

/// Helper function to set up a temporary directory for testing
fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

#[test]
fn test_detect_project_name_rust() {
    let temp_dir = setup_temp_dir();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"[package]
name = "my-rust-project"
version = "0.1.0""#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-rust-project");
}

#[test]
fn test_detect_project_name_nodejs() {
    let temp_dir = setup_temp_dir();
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(
        &package_json,
        r#"{"name": "my-node-project", "version": "1.0.0"}"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-node-project");
}

#[test]
fn test_detect_project_name_go() {
    let temp_dir = setup_temp_dir();
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module github.com/user/my-go-project\n\ngo 1.21").unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-go-project");
}

#[test]
fn test_detect_project_name_fallback() {
    let temp_dir = setup_temp_dir();
    // No config files, should use directory name

    let project_name = detect_project_name(temp_dir.path());
    // The temp directory name will be something like .tmpXXXXXX,
    // so we just verify it's not empty and not "unknown"
    assert!(!project_name.is_empty());
    assert_ne!(project_name, "unknown");
}

#[test]
fn test_detect_project_name_scala() {
    let temp_dir = setup_temp_dir();
    let build_sbt = temp_dir.path().join("build.sbt");
    std::fs::write(
        &build_sbt,
        r#"ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.3.0"

lazy val root = (project in file("."))
  .settings(
    name := "my-scala-project",
    libraryDependencies += "org.scalatest" %% "scalatest" % "3.2.15" % Test
  )"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-scala-project");
}

#[test]
fn test_detect_project_name_java_maven() {
    let temp_dir = setup_temp_dir();
    let pom_xml = temp_dir.path().join("pom.xml");
    std::fs::write(
        &pom_xml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
                             http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>my-java-maven-project</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
</project>"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-java-maven-project");
}

#[test]
fn test_detect_project_name_java_gradle() {
    let temp_dir = setup_temp_dir();
    let build_gradle = temp_dir.path().join("build.gradle");
    std::fs::write(
        &build_gradle,
        r#"plugins {
    id 'java'
    id 'application'
}

rootProject.name = "my-java-gradle-project"

group = 'com.example'
version = '1.0.0'

repositories {
    mavenCentral()
}

dependencies {
    testImplementation 'junit:junit:4.13.2'
}"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-java-gradle-project");
}

#[test]
fn test_detect_project_name_java_gradle_kts() {
    let temp_dir = setup_temp_dir();
    let build_gradle_kts = temp_dir.path().join("build.gradle.kts");
    std::fs::write(
        &build_gradle_kts,
        r#"plugins {
    kotlin("jvm") version "1.9.0"
    application
}

rootProject.name = "my-kotlin-project"

group = "com.example"
version = "1.0.0"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
}"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-kotlin-project");
}

#[test]
fn test_detect_project_name_elixir() {
    let temp_dir = setup_temp_dir();
    let mix_exs = temp_dir.path().join("mix.exs");
    std::fs::write(
        &mix_exs,
        r#"defmodule MyElixirProject.MixProject do
  use Mix.Project

  def project do
    [
      app: :my_elixir_project,
      version: "0.1.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    []
  end
end"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my_elixir_project");
}

#[test]
fn test_detect_project_name_python_pyproject() {
    let temp_dir = setup_temp_dir();
    let pyproject_toml = temp_dir.path().join("pyproject.toml");
    std::fs::write(
        &pyproject_toml,
        r#"[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "my-python-project"
version = "0.1.0"
description = "A sample Python project"
authors = [
    {name = "Author Name", email = "author@example.com"}
]
dependencies = [
    "requests>=2.25.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "black",
    "flake8",
]"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-python-project");
}

#[test]
fn test_detect_project_name_python_setup_py() {
    let temp_dir = setup_temp_dir();
    let setup_py = temp_dir.path().join("setup.py");
    std::fs::write(
        &setup_py,
        r#"from setuptools import setup, find_packages

setup(
    name="my-python-setup-project",
    version="1.0.0",
    description="A sample Python project using setup.py",
    author="Author Name",
    author_email="author@example.com",
    packages=find_packages(),
    install_requires=[
        "requests>=2.25.0",
    ],
    python_requires=">=3.8",
)"#,
    )
    .unwrap();

    let project_name = detect_project_name(temp_dir.path());
    assert_eq!(project_name, "my-python-setup-project");
}

#[test]
fn test_detect_project_name_invalid_directory() {
    let non_existent_path = Path::new("/this/path/does/not/exist");
    let project_name = detect_project_name(non_existent_path);
    assert_eq!(project_name, "exist");
}
