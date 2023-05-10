
# BkLog

[![version](https://img.shields.io/crates/v/bklog.svg?style=flat-square)](https://crates.io/crates/bklog)
[![docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/bklog)

An attempt at a WAL-like corruption resilient storage for IoT local backlogging.

## Motivation

This library is conceived for the purpose of saving data on local storage, should a remote main
storage system be temporarily unavailable. Once it returns, the backlog is then read and
consumed as it is being written to said main storage.

Its main use is aimed at applications that would otherwise loose data. For example IoT devices
that can be potentially and temporarily disconnected from the storage they are supposed to log
data to.

## Warning

Current approach does not avoid data loss. Unfortunately, there is no way to guarantee that
other than battery with powered underlying storage hardware or a UPS. This library is meant to
merely keep the log from corrupting beyond the ability of an automated recovery, all the while
being explicit about what was lost, so it can be reported as an error.

## Usage Example

TBD
