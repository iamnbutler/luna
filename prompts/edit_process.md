Follow this process before you make each edit:

1. Think about the current problem. If your last attempt created errors or test failures, explain to me what you think went wrong and how you will solve it.
2. If you aren't confident about a solution, come up with 2-3 ways you _might solve it_ and share them with me.
3. You choose an option if there are multiple options. Don't wait for my feedback.
4. Create an implementation plan:
  4. 1. Ask yourself: "Can I use test driven development for this solution? Will the tests that come out of this be useful and actually catch issues introduced to the system? Or are they contrived so I can say I'm doing test driven development?"
  4. 2. Either implement your plan or break the problem down into smaller pieces
5. Execute your plan. Make any edits you need to make. Use cargo check, cargo test, and your diagnostics tool to check for issues.
6. If it is helpful, use `cargo run` to run the app (with an automated max 30-second kill switch to kill the task) to be able to access any runtime-specific logs.
7. Once this logical step of your plan is complete, error free, with passing tests, write a commit message and commit your changes to the current branch. If you are correcting issues from your previous plan, don't just write "implement whatever the previous plan was", write a commit message explaining what you are fixing, and what went wrong with the previous step. Don't limit yourself to 100 characters or whatever, I'd rather have a clear message than a short one.
8. Finally, if you have finished all your steps and completed the original task I gave you, let me know you are done.
