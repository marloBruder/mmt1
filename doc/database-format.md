# Database format

An .mm.sqlite file should have the following tables and fields:

## Tables

- Theorem
  - DB placement (ID)
  - Name
  - Description
  - Hypotheses
    - List of Hypothesis/Strings
  - Assertion
  - Distinct Variables
  - Proof
    - (Could be divided further)
- Variable Statement
  - DB placement (ID)
  - Name
  - Kind
  - Variable
- Comment
  - DB placement (ID)
  - Text
- In progress Theorem
  - Name
  - Text
- Token
  - ASCII representation
  - HTML/visual representation

## Required operations

(All operations regarding theorems, variable statements or comments should be able to uphold an ordering based on their DB placement ID)

- Add/Edit/Get Theorem with name/number
- List Theorems n to m and the preceeding comments
- Search Therorems with filters/orderings
- Add/Edit/Get In-progress theorem
- ...
