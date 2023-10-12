
<h3 align="center">
Crolk is a simple gui clock, stopwatch, timer, alarm app made with gtk-rs.
</h3>

# Install
No binary installations avaliable

## Linux
- there is one optional dependency `dracula-icons-git` that is used for the icon

### Build and install from the AUR
```
yay -S crolk-git
```

### Manual Build and install
```
git clone https://github.com/crolbar/crolk
cd crolk
cargo build --frozen --release
```

then you can cp the binary in the $PATH
```
sudo cp target/release/crolk /usr/bin/crolk
```

also there is an desktop entry in `resources` so you can cp that too
```
sudo cp resources/crolk.desktop /usr/share/applications
```



## Windows
no
