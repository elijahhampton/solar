//! Common data datastructures.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/danipopes/rsolc/main/assets/logo.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/danipopes/rsolc/main/assets/favicon.ico"
)]
#![warn(unreachable_pub, rustdoc::all)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use smallvec as _;

pub mod fx;
