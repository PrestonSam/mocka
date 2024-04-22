# Mockagen

`Error` shouldn't need to take a lifetime parameter. Look into cloning data until you own it.

I should look into enriching the errors that come out of the packer. At the moment I can see the providence of the error with relation to the input file, but I can't see any stack trace relating to the packer itself. Ideally I should be able to wrap the errors coming out with the name of the function that called it. Of course it seems rather silly to painstakingly re-implement such a basic language feature...

I could always throw the error and have done with it? Then again that'd reveal a lot of internal workings that the user shouldn't have to look at.


What sort of type signature should I create for the evaluator?

In the last implementation, I returned a hash of functions with their names as the key (although I believe ultimately this was unpacked and the function pointers were inserted directly into the mockadoc generated code).
Of course the functions that have a USING clause must accept arguments, too.
Presumably that means I'd have to order the rules by their dependencies?

In my last method, I generated and returned lambdas. That's a very functional approach.
What would it be like if I used structs this time to describe what functions to call?
Would that be simpler? Would it be faster?
I'm not confident it would be, tbh.
Although it'd definitely be easier to serialise.



I need to separate out the primitive values from the higher order values.
This should make it much clearer to tell what's going on, when I'm evaluating the AST.
