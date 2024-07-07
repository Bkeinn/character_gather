Pre alpha

gather command
```
./target/debug/character_gather gather -a "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z, ,." -i ../smalltest.txt -o full2.h5  --offset-back 3 --offset-front 3
```
normalize command

```

```

# HDF5 file
## Absolute Data
- y/vertical = Base character
- x/Horizontal = Found character
- z/depth = distance
a | b | 0 -> If you are character a and go 0 - offsetback back you find character b n times
