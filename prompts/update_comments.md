Comments should be written from the perspective of a senior Rust developer, using clear and concise language. Never write comments that just restate what the code is doing.

In functions, there should only ever be comments that explain non-obvious behavior or for wayfinding in very large functions.

Based off of the target audience mentioned above, review any code and documentation comments you added in this change, as well as in the areas this change touched.

- is this a comment // and is it saying with the code is doing, not explaining something non-obvious, or providing a key point that future developers will need to understand the code?
- is this a comment // and is it a useful placeholder for future functions or upcoming changes?
- is this a doc comment ///? If so, is it providing useful context, and writing at the target audience mentioned above? (write in a technical manner as a per of and targeted at senior Rust developers.)
- is this any kind of comment, and it is describing a behavior or fact that is no longer true or accurate? If so, update or remove it based on your judgement and the above context.

Finally, as you review the code, look at other comments you encounter and update them as needed based on the criteria mentioned above.

Let’s update any comments as needed before we merge the PR.
