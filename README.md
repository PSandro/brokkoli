
## Raspberry Pi 4 setup

### setup i2c
```
sudo raspi-config
-> Interface Options
-> I2C
-> enable
-> reboot
```

### list i2c devices
```
sudo i2cdetect -y 1
```

### setup [camera-streamer](https://github.com/ayufan/camera-streamer)
check docs on camera-streamer


