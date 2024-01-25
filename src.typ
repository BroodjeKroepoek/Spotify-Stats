#import "@preview/codly:0.2.0": *

#set heading(numbering: "1.1.")
#set page(numbering: "1 / 1")

#show: codly-init.with()

#let icon(codepoint) = {
  box(height: 0.8em, baseline: 0.05em, image(codepoint))
  h(0.1em)
}

#codly(languages: (rust: (
  name: "Rust",
  icon: icon("assets\brand-rust.svg"),
  color: rgb("#CE412B"),
)))

#outline(indent: auto)

#pagebreak()

= trait FromFolderJson

```rust
pub trait FromFolderJson {
    fn from_folder_of_json<P>(folder: P) -> Result<Self>
    where
        Self: Sized,
        P: AsRef<Path>;
}
```

= struct EndStream

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct EndStream { /*...*/ }
```

= unit struct EndStreamContainer

```rust
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct EndStreamContainer(Vec<EndStream>);
```

== implementations
#enum(
  enum.item(1)[
    ```rust
    impl EndStreamContainer {
      fn new() -> Self { /*...*/ }
    }
    ```
  ],
  enum.item(2)[
    ```rust
    impl FromFolderJson for EndStreamContainer { /*...*/ }
    ```
  ],
  enum.item(3)[
    ```rust
    impl IntoIterator for EndStreamContainer { /*...*/ }
    ```
  ],
  enum.item(4)[
    ```rust
    impl FromIterator<EndStream> for EndStreamContainer { /*...*/ }
    ```
  ]
)

= enum EndStreamKind

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum EndStreamKind {
  EndSong,
  EndEpisode,
  EndVideoOrElse,
}
```

= struct EndStreamWithKind
```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamWithKind {
    pub kind: EndStreamKind,
    pub end_stream: EndStream,
}
```

== implementations
#enum(
  enum.item(1)[
    ```rust
    impl From<EndStream> for EndStreamWithKind { /*...*/ }
    ```
  ]
)

= unit struct EndStreamWithKindContainer
```rust
#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct EndStreamWithKindContainer(pub Vec<EndStreamWithKind>);
```

== implementations

#enum(
  enum.item(1)[
    ```rust
    impl EndStreamWithKindContainer {
        fn new() -> Self { /*...*/ }
    }
    ```
  ],
  enum.item(2)[
    ```rust
    impl FromFolderJson for EndStreamWithKindContainer { /*...*/ }
    ```
  ],
  enum.item(3)[
    ```rust
    impl IntoIterator for EndStreamWithKindContainer { /*...*/ }
    ```
  ],
  enum.item(4)[
    ```rust
    impl FromIterator<EndStreamWithKind> for EndStreamWithKindContainer { /*...*/ }
    ```
  ],
  enum.item(5)[
    ```rust
    impl From<EndStreamContainer> for EndStreamWithKindContainer { /*...*/ }
    ```
  ],
)







