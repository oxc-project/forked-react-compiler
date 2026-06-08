
## Input

```javascript
import {useState} from 'react';

const bar = () => ({data: null});

export const useFoot = () => {
  const [, setState] = useState(null);
  try {
    const {data} = bar();
    setState({
      data,
      error: null,
    });
  } catch (err) {
    setState(_prevState => ({
      loading: false,
      error: err,
    }));
  }
};

```


## Error

```
Found 1 error:

Invariant: Expected all references to a variable to be consistently local or context references

Identifier <unknown> err$7 is referenced as a context variable, but was previously referenced as a local variable.

error.bug-invariant-local-or-context-references.ts:16:13
  14 |     setState(_prevState => ({
  15 |       loading: false,
> 16 |       error: err,
     |              ^^^ this is local
  17 |     }));
  18 |   }
  19 | };
```
          
      