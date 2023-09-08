# Papyrus

Papyrus is a scripting language in which you can modify a bunch of canvas. These canvas can then be saved in several file formats including images (png and jpeg) or video (soon).

The lexing and parsing are powered by [Neoglot](https://github.com/Mkdirs/neoglot-lib).

# Download

Download the latest release [here](https://github.com/Mkdirs/papyrus/releases/latest).

# Specifications

## Types

* color:    A 64-bits unsigned integer representing a color
* int:      A 32-bits signed integer
* float:    A 32-bits floating point number according to the IEE 754 single precision
* bool:     A boolean

A color literal is a hexadecimal number starting with '#' and consisting of 8 digits. 2 for each rgba channel.
\
Ex: `#ff00ffff` is red: 255, green: 0, blue: 255, alpha: 255 which is purple.

An int literal match the regular expression `-?[0-9]+`.\
Ex: `10`, `20`, `-58`, `0` etc.

A float literal match the regular expression `-?[0-9]+.[0-9]+`.\
Ex: `0.0`, `-12.45`, `0.003` etc.

A bool literal is `true` or `false`.

## Variables

To declare a variable you must follow the following:\
name:type
```
// Declaring a variable named a of type int
a:int;
```

> **Note**: '//' allow to make a single line comment.

You can initialize a variable while declaring it:
```
a:int = 10;
```

To assign a value to a variable:
```
a = -58;
```

You cannot assign a value to an undeclared variable. The type of the value must match the variable type: you cannot assign a float to an int for instance.

> **Note**: as you have seen every instruction ends with ';'.

## Functions

To declare a function:
```
// Declaring a function foo with parameters bar and baz
// The function returns a color
def foo(bar:int, baz:float) : color{
    // very intensive computations...
    return #ffffffff;
}
```

You can ommit the return type
```
// foo takes an int but returns nothing (void)
def foo(bar:int){
    // code...
}
```

To call a function:
```
// We call the first function and store the result
b:color = foo(0, 0.0);
```

You cannot declare two functions with the same signature (the same name and parameters type).\
The return type is not part of a function signature.

> **Note**: All your code must be inside functions.\
You must have a function called 'main' which is the entry point of your script.


## Built-in functions

`create_canvas(w, h)`\
`w:int`\
`h:int`\
`returns void`\
Creates a canvas of width `w` and height `h` and push it on the canvas stack.

`save_canvas()`\
`returns void`\
Pops the top of the canvas stack and saves it for later.

`put(x, y, col)`\
`x:int`\
`y:int`\
`col:color`\
`returns void`\
Sets the color of the pixel (`x`, `y`) of the top of the canvas stack as `col`.

`fill(col)`\
`col:color`\
`returns void`\
Fills entirely the top of the canvas stack with the color `col`.

`sample(x, y)`\
`x:int`\
`y:int`\
`returns color`\
Sample the color of the pixel (`x`, `y`) of top of the canvas stack. 

`width()`\
`returns int`\
Gives the width of the top of the canvas stack.

`height()`\
`returns int`\
Gives the height of the top of the canvas stack.

`float(x)`\
`x:int`\
`returns float`\
Cast an int into a float.

`int(x)`\
`x:float`\
`returns int`\
Cast a float into an int.

`red(col)`\
`col:color`\
`returns int`\
Gives the red value of a color.

`green(col)`\
`col:color`\
`returns int`\
Gives the green value of a color.

`blue(col)`\
`col:color`\
`returns int`\
Gives the blue value of a color.

`alpha(col)`\
`col:color`\
`returns int`\
Gives the alpha value of a color.

## Subdividing a canvas

```
subcanvas(x, y, w, h){
    //...
}
```
`x:int`\
`y:int`\
`w:int`\
`h:int`\
Creates a new canvas of size `w`x`h` and push it on top of the canvas stack.\
At the end of the block the contents of the canvas are copied to the region starting at (`x`, `y`) of dimensions `w`x`h` of the second canvas on the stack.\
The first canvas is then removed of the stack.


## Control flow structures

`if`/`else if`/`else` work like other languages
```
if(condition1){
    //...
}else if(condition2){
    //...
}else{
    //...
}
```

The only loop is `while`:
```
while(condition){
    //...
}
```

## Operators

You have the common operators on numbers:
* Addition: `+`
* Substraction: `-`
* Multiplication: `*`
* Division: `/`
* Exponentiation: `^`

Comparison:
* Greater than: `>`
* Lower than: `<`
* Equals: `==`
* Not equals: `!=`
* Greater than or equals: `>=`
* Lower than or equals: `<=`

And the boolean complement: `!`

## Importing

You can import functions from other scripts by using the keword `import`.\
Functions that can be imported use the visibility modifier `pub`.

File draw.pprs:
```
pub fn square(x:int, y:int, w:int, h:int, col:color){
    subcanvas(x, y, w, h){
        fill(col);
    }
}
```

File main.pprs:
```
import "draw";

def main(){
    create_canvas(100, 100);
    fill(#ffffffff);
    draw.square(0, 0, 100, 25, #ff0000ff);
    save_canvas();
}
```

The file path you give in import must not contain any file extension: It will append '.pprs' automatically.\
You can give an alias to the script you import with the keyword `as`:
```
import "foo/author/draw";
import "draw" as my_draw;

def main(){
    draw.foo();
    my_draw.bar();
}
```
That also allow importing two files that have the same name.

# CLI

Running a script: `papyrus run <file>`.\
More informations on the commands can be found by running `papyrus help`.