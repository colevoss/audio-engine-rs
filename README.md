## Making mono file stereo

```
source_channel_count = 1;
target_channel_count = 2;

current_frame = 0;

frames = [[0,1,2,3,4,5,6]]
```

when getting the first (left) frame, we know we can just get frames[0][0]
When getting the first right channel (current_frame = 1), we can get frames[current_frame % source_channel_count][current_frame - (target_channel_count - source_channel_count)]
