# Tasks: git-status-directory-counts

## Task 1: Add DirectoryCount struct and extend GitStatus

Add the `DirectoryCount` struct with `directory: String` and `count: u32` fields. Add `directory_counts: Vec<DirectoryCount>` to the existing `GitStatus` struct. Update all existing construction sites of `GitStatus` in `parse_porcelain_v2` to include the new field (initially as an empty vec).

## Task 2: Extract file paths and aggregate per-directory counts

Add a `parent_directory` helper function. During the `parse_porcelain_v2` loop, extract the file path from each changed-file line (ordinary, renamed, untracked, unmerged) and accumulate counts in a `HashMap<String, u32>` keyed by parent directory. After the loop, convert to a sorted `Vec<DirectoryCount>` (descending by count) and set it on the `GitStatus`.

## Task 3: Add unit tests for directory counts

Add tests to the existing `mod tests` block: clean repo returns empty directory_counts, files across multiple directories produce sorted entries, root-level files grouped under ".", renamed file uses destination path, mixed change types in the same directory combine counts.
