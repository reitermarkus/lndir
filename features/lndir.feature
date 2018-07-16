Feature: default behaviour

  Scenario: Linking a directory into another one
    Given the directory structure
      """
        .
        ├── A.d
        │   ├── dir.d
        │   └── file
        └── B.d
      """
    When I run `lndir A.d B.d`
    Then the resulting directory structure is
      """
        .
        ├── A.d
        │   ├── dir.d
        │   └── file
        └── B.d
            ├── dir.d → ../A.d/dir.d
            └── file → ../A.d/file
      """

  Scenario: Linking multiple directories into another one
    Given the directory structure
      """
        .
        ├── A.d
        │   ├── dir1.d
        │   └── file1
        ├── B.d
        │   ├── dir2.d
        │   └── file2
        └── C.d
      """
    When I run `lndir A.d B.d C.d`
    Then the resulting directory structure is
      """
        .
        ├── A.d
        │   ├── dir1.d
        │   └── file1
        ├── B.d
        │   ├── dir2.d
        │   └── file2
        └── C.d
            ├── dir1.d → ../A.d/dir1.d
            ├── dir2.d → ../B.d/dir2.d
            ├── file1 → ../A.d/file1
            └── file2 → ../B.d/file2
      """

  Scenario: Linking a directory containing a symlink
    Given the directory structure
      """
        .
        ├── A.d
        │   ├── file1
        │   └── file2 → file1
        └── B.d
      """
    When I run `lndir A.d B.d`
    Then the resulting directory structure is
      """
        .
        ├── A.d
        │   ├── file1
        │   └── file2 → file1
        └── B.d
            ├── file1 → ../A.d/file1
            └── file2 → file1
      """
