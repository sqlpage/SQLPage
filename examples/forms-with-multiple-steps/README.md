# Forms with multiple steps

Multi-steps forms are forms where the user has to go through multiple pages
to fill in all the information.
They are a good practice to improve the user experience
on complex forms by removing the cognitive load of filling in a long form at once.
Additionally, they allow you to validate the input at each step,
and create dynamic forms, where the next step depends on the user's input.

There are multiple ways to create forms with multiple steps in SQLPage,
which vary in the way the state of the partially filled form
is persisted between steps.

This example illustrates the main ones.
All the examples will implement the same simple form:
a form that asks for a person's name, email, and age.

## [Storing the state in the database](./database/)

You can store the state of the partially filled form in the database,
either in the final table where you want to store the data,
or in a dedicated table that will be used to store only partial data,
allowing you to have more relaxed column constraints in the partially filled data.

 - **advantages**
   - the website administrator can access user inputs before they submit the final form
   - the user can start filling the form on one device, and continue on another one.
   - the user can have multiple partially filled forms in flight at the same time.
 - **disadvantages**
   - the website administrator needs to manage a dedicated table for the form state
   - old partially filled forms may pile up in the database

## [Storing the state in cookies](./cookies/)

You can store each answer of the user in a cookie,
using the 
[`cookie` component](https://sql.datapage.app/component.sql?component=cookie#component).
and retrieve it on the next step using the
[`sqlpage.cookie` function](https://sql.datapage.app/functions.sql?function=cookie#function).

 - **advantages**
   - simple to implement
   - if the user leaves the form before submitting it, and returns to it later,
     the state will be persisted.
   - works even if some of the steps do not use the form component.
 - **disadvantages**
   - the entire state is re-sent to the server on each step
   - the user needs to have cookies enabled to fill in the form
   - if the user leaves the form before submitting it, the form state will keep being sent to all the pages he visits until he submits the form.

## [Storing the state in hidden fields](./hidden/)

You can store the state of the partially filled form in hidden fields,
using `'hidden' as type` in the [form component](https://sql.datapage.app/component.sql?component=form#component).

 - **advantages**
   - simple to implement
   - the form state is not sent to the server when the user navigates to other pages
 - **disadvantages**
   - the entire state is re-sent to the server on each step
   - you need to reference all the previous answers in each step
   - no *backwards navigation*: the user has to fill in the steps in order. If they go back to a previous step, you cannot prefill the form with the previous answers, or save the data they have already entered.
