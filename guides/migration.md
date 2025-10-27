# mmj2/yamma to mmt1 migration guide

## Welcome

Welcome to the mmj2/yamma to mmt1 migration guide. This guide will teach you in great detail how to use mmt1 with the assumption that you are already somewhat familiar with mmj2 or yamma. If that is not the case I recommend watching David A. Wheelers video [Introduction to Metamath and mmj2](https://www.youtube.com/watch?v=Rst2hZpWUbU) and coming back to this guide afterwards. For instructions on how to install mmt1 please see [README.md](../README.md).

## Basic Navigation

If you open mmt1, you will be greeted by two navigation menus: The title-bar at the top and the side-bar on the left. The title-bar has four sections:

1. Under `File`, you can open and close folders, save the currently open file and exit mmt1.
2. Under `Editor`, you will find commands related to the editor, such as unify, format, renumber and add to database.
3. Under `Metamath` you can create, open and close metamath databases.
4. Under `About` you will find information about mmt1 and the ability to update the program.

After opening mmt1, you are usually going to want to first open the metamath database you want to work on and the folder containing your `.mmp` files. After clicking `Metamath > Open Metamath Database` and selecting your `.mm` file you will be brought to the database opening screen. Here you can watch mmt1 make progress parsing, verifying and calculating relevant data about your `.mm` file. Note that you can click the `Confirm Open Database` button before mmt1 has finished calculating the parse trees. At this point you can already start browsing your database and `.mmp` files. You however cannot use features that rely on the parse trees, such as the unifier, proof generator or unicode preview colorer. If you want to see the progress towards calculating all parse trees after leaving the database opening screen, you can view it in the settings tab.

Let's move on to the side-bar. It is made up of three sections that you can switch between and one button that will open the settings tab. The sections are the following:

1. In the `Theorem Explorer` section you can browse your metamath database either by ...
   - ... pressing the `New Theorem Explorer` button, which will open a new tab at page 1 of the theorem explorer (similar to the theorem explorer from the web pages, the biggest difference being that here comments, constants, variables and floating hypotheses are shown as well),
   - ... using the quick search to quickly find theorems based on their label or
   - ... browsing the header explorer, which is like a file explorer only with headers instead of folders and statements instead of files. (Double) clicking any statement will open it in a new tab. Or you can right click a statement or header and press `Open In New Theorem Explorer`, which will open a new theorem explorer tab on the right page and show you the statement or header.
2. In the `Search` section you can query the theorem of your metamath database based on their label, their parse trees, their axiom and definition dependencies and their theorem type. More on how the `Search by Parse Tree` works later.
3. In the `File Explorer` section you can browse your opened folder. Clicking a file will open it in a new editor tab. Additionally you you can create, rename and delete files or folders (moving them is not yet supported as of mmt1 `v1.0`).

Clicking the currently open section will collapse the side-bar. Additionally you can change the width of the side-bar by dragging the divider between it and the main section.

## Tabs

When opening statements or files from the side-bar with only a single click the tabs opened will be temporary. That means that as soon as you open the next tab, the old one will be closed. You can tell that a tab is temporary by it's _italic_ name, double clicking which will make the tab permanent. Alternatively you can double click statements or files, which will open them permanently right away.

Sometimes pages will have links to other pages within them. Clicking them will change the tab to the linked page. If you want the page to be instead opened in a new tab, you can middle mouse click the link. Tabs store a history of the pages you have visited within them. You can go back and forward in that history using the arrow buttons on the left side of the tab-bar. Tabs can be closed using the `Ctrl + W` shortcut.

## The editor

By now you should be able to easily navigate mmt1 and know where everything is. So let's move on the proof assistant part of mmt1.

Within the editor you can edit `.mmp`, `.mm` or other files which contents can be represented in `utf-8`. mmt1 uses monaco editor as it's editor, which is a standalone version of vscodes editor. Because of this you can edit large `.mm` files right within mmt1, no matter their size (although editing your currently open `.mm` file will disable the `Add to Database` feature until you reload the database). Additionally you also have access to vscodes very useful shortcuts. Here is a collection of shortcuts I often use while editing `.mmp` files:

- Holding `Shift + Alt` and pressing either `UpArrow` or `DownArrow` will copy the line the cursor is on (or the lines you have selected) up or down respectively. This is very useful when you want to add a step that is very similar to another existing step.
- Holding just `Alt` and pressing either `UpArrow` or `DownArrow` will instead move the line the cursor is on (or the lines you have selected) up or down respectively. This is very useful when you want quickly move steps.
- Selecting something and then pressing `Ctrl + D` will insert another cursor at the next point matching your selection. By pressing `Ctrl + K` and then `Ctrl + D` you can skip an instance of your selection. This is very useful for substituting symbol or work variables.
- Alternatively you can also hold down `Alt` and then add or remove cursors at will with your mouse left click.
- You can delete the line the cursor is on (or the lines you have selected) by pressing `Ctrl + Shift + K`.
- Pressing `Ctrl + Enter` will add a new line after the current one, even if your cursor is in the middle of it.

For more useful shortcuts just look up vscodes shortcuts. Not all of them will work as some are specific to vscode, but most related to editing text should also work within mmt1.

mmt1 also has some shortcuts of it's own. Mainly:

- `Ctrl + S` which saves the currently opened file. (By default this will also format `.mmp` files, but this can be turned off in the settings.)
- `Ctrl + U` which triggers the unifier.
- `Ctrl + R` which renumbers all steps.
- `Shift + Alt + F` which formats `.mmp` files.

Unfortunately as of mmt1 `v1.0` you can only trigger these shortcuts while focused on the editor (as in that your cursor is in the editor).

## The extended mmp syntax

If you have opened one of your `.mmp` files written with mmj2 you might have noticed that mmt1 will show you an error. This is because mmt1 uses an extended version of yammas syntax, which itself has a slightly different header syntax from mmj2. mmt1 extends yammas syntax so that you are able to not just express theorems, but also axioms, floating hypotheses, variables, constants, comments and headers. Here's how you can do that:

### Headers

```
$header 1.2.3 Your header title

* Your (optional) header description
```

As you can see, you declare that your `.mmp` file represents a header using the `$header` statement. It is followed by the path of your header and then the title of the header. The first comment will additionally become the description of the header. You do not have to worry about the formatting of your description, as mmt1 will wrap it automatically. Descriptions can be given multiple paragraphs, by leaving a gap of 2 new-lines or more between words. The gaps will be collapsed to 2 new-lines when you add your header to the database. Headers are the only statement type that cannot use `$locateafter` (or it's variants), as their position in the database is determined by their header path.

### Comments

```
* Your comment text
```

If a `.mmp` file contains nothing but comments and optionally a `$locateafter` variant, then it represents a comment. Unlike the descriptions of headers (or theorems), comments are not wrapped when added to the database. This gives you more control over the formatting of your comment, which can be useful if you are adding a `$j` comment for example.

### Constants

```
$c const1 const2 ...
```

If a `.mmp` file contains nothing but a `$c` statement and optionally a `$locateafter` variant and comments, then it represents a constant statement. As with constant statements in `.mm` files you can add multiple constants per statement.

### Variables

```
$v var1 var2 ...
```

If a `.mmp` file contains nothing but a `$v` statement and optionally a `$locateafter` variant and comments, then it represents a variable statement. As with variable statements in `.mm` files you can add multiple variables per statement.

### Flaoting Hypotheses

```
$f label typecode var
```

If a `.mmp` file contains nothing but a `$f` statement and optionally a `$locateafter` variant and comments, then it represents a floating hypothesis statement. Unlike floating hypothesis statements in `.mm` files, the label comes after the `$f`. This is so that all non comment and non proof step statements start with a `$`.

### Axioms

```
$axiom ax-example

* Your (optional) axiom description

h1::ax-example.1    |- ph
h2::ax-example.2    |- ( ph -> ps )
qed::               |- ps
```

If your mmp file has an `$axiom` statement, then it represents an axiom. The description is handled in the exact same way as those of headers. axiom `.mmp` files may only have hypotheses or qed proof steps and the qed steps ref must be left empty. Since mmt1 can only be used for creating grammatical metamath databases, it is required that all expressions are parsable using the databases grammar (unless you are adding a syntax axiom of course).

### Theorems

```
$theorem example

* Your (optional) theorem description

* (optional)
$allowdiscouraged

* (optional)
$allowincomplete

* (optional)
$d ...

* Your proof steps
h1::example.1       |- ph
...
qed:1:idi           |- ph
```

If your mmp file has an `$theorem` statement, then it represents an theorem. The description is handled in the exact same way as those of headers. The proof steps work almost identical to mmj2 or yamma. There are however a few differences:

- mmt1 features an updated work variable syntax to eliminate the potential for name conflicts. Givin any variable, appending a `$` and number will give you a valid work variable. So in `set.mm` examples of work variables would for be `ph$1`, `y$2` or `C$3`.
- When there are no `:` at the beginning of the line then the token will be interpreted as the step name. This differs from mmj2 or yamma where it would be interpreted as the step ref, that is unless it starts with an `h`, in which case it would be interpreted as the step name. I decided to make this change so that all theorems are treated the same, regardless of whether they start with `h`, which to me seemed like an inconsistency.
- Some unifier features are only activated when you put a `!` at the front of the line (specifically the advanced hypothesis finding feature). This works the same as in mmj2, but differs from yamma, where such syntax does not exist.
- You can use `?` for a hypothesis that is not yet known. Once again this works the same as in mmj2, but differs from yamma, where you cannot do that.

Another difference to yamma is that in mmt1 you can have more than 2 variables per `$d` statement.

When mmt1 encounters an incomplete proof (one where a label is replaced by a `?`) while verifying the database, it does not give you an error. Instead it will mark those theorems and all theorems dependent on them as incomplete, which means that by default the unifier will ignore them and you will get an error when trying to use them. This can be deactivated using an `$allowincomplete` statement.

Note also that you won't get an error if you add `$v` or `$f` statements to theorems (or axioms). Initially I wanted to allow you to use temporary variables and temporary floating hypotheses in proofs and axioms. Implementing this has however turned out to be much harder than initially tought with my current architecture.

### Locateafter

By now you might be wondering what I mean with the term "`$locateafter` variants". Since mmt1 allows you to add statements to the database automatically, you need more precision when specifying where in the database your statement is supposed to be inserted. For this I have implemented more `$locateafter` variants. They are the following:

- `$locateafterstart`: Must not be followed by any token. Locates the statement at the start of the database.
- `$locateafterheader`: Must the followed by the path of a header (Example: 1.2.3). Locates the statement after the specified header.
- `$locateaftercomment`: Must the followed by the path of a comment (Example: 1.2.3#4, where 1.2.3 is the path to the parent header of the comment, and the comment is the 4th comment in that header). Locates the statement after the specified comment.
- `$locateafterconst`: Must the followed by a constant. Locates the statement after the constant statement containing the constant.
- `$locateaftervar`: Must the followed by a variable. Locates the statement after the variable statement containing the variable.
- `$locateafter`: Must be followed by the label of a floating hypothesis, axiom or theorem. Locates the statement after the specified statement.

The `$locateafter` variants can be used in all `.mmp` files that don't represent headers, as their location is determined from their header path, as mentioned above.

## The Unicode preview

Now that you know the syntax that mmt1 uses, you should be able to create your first proof. To aid you during proof development, mmt1 has a build in Unicode preview. It can be activated using the dropdown on the right side of the tab-bar. You have four options: To hide the unicode preview, to pin it to the right or bottom side of the editor or to pop it out into an external window. The last option is of course the best one if you have a second monitor or a single monitor connected to a laptop.

The unicode preview does a lot more than just showing you a representation of your proof in Unicode. It also shows you axiom and definition dependencies, helping you keep track of them. Additionally, the proof lines in the Unicode preview are colored to convey information to you. The meaning of the colors are:

- <span style="color:#003d30">■</span> Darker green: This line is correct.
- <span style="color:#005030">■</span> Lighter green: This line and all lines it depends upon are correct.
- <span style="color:oklch(25.8% 0.092 26.042)">■</span> Darker red: There is an error in this cell
- <span style="color:oklch(39.6% 0.141 25.723)">■</span> Lighter red: This line will be removed after unifying.
- <span style="color:oklch(28.2% 0.091 267.935)">■</span> Dark blue: The unifier will make a change in this cell.

(Github unfortunately won't color the squares above correctly. To see the colors, please view this document in a non Github markdown renderer like vscode or view the colors within mmt1 as mentioned below)

You can deactivate the coloring of the Unicode preview (and also lookup the meanings again) in the settings. Another thing the Unicode preview shows you is what the unifier will do. This allows you to for example quickly try out different formulas to see which one the unifier finds an existing theorem for, without having to trigger the unifier each time. This too can be deactivated in the settings if you just want a pure Unicode preview.

Using the indention is another way to extract information out of the Unicode preview. If you for example want to quickly remind yourself which proof steps you have completed but not yet used for anything else, you can look for proof steps with an indention of 1, since that indicates that no other steps depends on them.

## Search by parse tree

While developing proofs using mmt1, you are bound to need to query the database at some point. This is where mmt1s main search feature `Search By Parse Tree` comes in. It allows you to query the database based on the parse trees of statements.

As an input it takes a series of conditions, each of which is evaluated independently. If all conditions match a theorem, the theorem is added to the search result. Each condition takes three parameters:

1. Where to look for the query parse tree. This can either be `any hypothesis`, `all hypotheses`, `the assertion`, `any hypothesis or the assertion` or `all hypotheses and the assetion`.
2. Whether the theorems parse tree should `match` or `contain` the query parse tree.
3. The `query parse tree` itself. Note two things here: That variables will match any variables of their type (but the same two query variables has to match the same two underlying variables) and that you can you work variables to represent an arbitrary parse tree.

Let's look at some examples:

- The query parse tree `|- ( ph$1 -> ph$2 )` with the second parameter set to `match` will find all parse trees that have an implication as the outer most syntax axiom.
- The query parse tree `wff ( ph$1 -> ph$1 )` with the second parameter set to `contain` will find all parse trees that have an implication somewhere in them where both wffs are the same.
- The query parse tree `class _V` with the second parameter set to `contain` will find all parse trees that make use of the `_V` class.
- The query parse tree `|- ph` with the second parameter set to `match` will find all parse trees that are just a `wff` variable, no matter which variable.

As you can see, whenever the second parameter is set to `match` your query parse tree is gonna want to start with a logical typecode and if it is set the `contain` your query parse tree is gonna want to start with a syntax typecode.

Within a condition all checks are done indepedent of another. So the query (`all hypotheses`, `match`, `|- ph`) will return a potential theorem with two hypotheses `|- ph` and `|- ps`. WARNING: This implementation detail is likely to change in the future, that is if I figure out an efficiant way to implement it.

As you might be able to guess right now, `Search By Parse Tree` can be quite useful for finding parse trees within singular expressions, but is not very good at finding theorems where need substitutions to be consistant accross multiple expressions. Seaching even for simple theorems (like `ax-mp`) is quite difficult. This is a big weakness of `Search By Parse Tree` central to it's design and will be hard to fix. This is why I'm already planning on implementing metamath-lamps `Seach by Pattern` as one of the first things after the launch of `v1.0`.

## Add to database

The last step after creating your proof (or statement) is to add it to the database. Luckily mmt1 can do that for you with just a few simple clicks. Navigate to `Editor > Add to Database` in the title-bar, which will bring you to the "Add to database" screen. Here you can see the exact change mmt1 is making to the database using monaco editors diff view. You can also choose the proof format, the default value of which can be changed in the settings. If you accidentally scroll away from the change mmt1 is making, you can use the `Scroll to Change` button to get back to it. If you are adding HTML to the database that is not on the [HTML whitelist](security.md), you will be warned here.

There is one limitation when adding statements to the database, and that is that you cannot insert statements between two theorems sharing a scope. If you try to do that, mmt1 will ask you to insert the statement manually, after which you have to reload the database.

## Conclusion

This concludes the mmj2/yamma to mmt1 migration guide. If you have any further questions about mmt1 feel free to create an issue on [Github](https://github.com/marloBruder/mmt1) or send an email to the [official metamath mailing list](https://groups.google.com/g/metamath). I'll be happy to help you out :).
