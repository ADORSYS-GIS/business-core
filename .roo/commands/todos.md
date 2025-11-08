# Process TODOs Command

## Purpose
Find and resolve all TODO comments in the specified file(s) by implementing the requested changes, fixes, or improvements.

## Instructions
1. Search for all comments containing `TODO`, `Todo`, or `todo` (case-insensitive)
2. For each TODO found:
   - Read and understand the instruction or issue described
   - Analyze the surrounding code context
   - Implement the necessary changes to resolve the TODO
   - Remove or update the TODO comment once resolved
3. Verify that all changes work correctly together
4. Report what was done for each TODO item

## Usage
Simply reference this command with the target file(s):
- Single file: "Process TODOs in `src/main.rs`"
- Multiple files: "Process all TODOs in the `src/repository/` directory"
- Entire project: "Process all TODOs in the codebase"

## Examples
- "Use `.roo/commands/todos.md` on `business-core-postgres/src/test_helper.rs`"
- "Apply todos.md to all files in `business-core-db/src/models/`"
- "Process all TODO comments in the project"

## Notes
- The assistant will read files to find TODOs before making changes
- All changes will be explained clearly
- If a TODO is unclear or needs more context, the assistant will ask for clarification