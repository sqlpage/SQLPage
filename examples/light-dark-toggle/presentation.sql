SELECT 'dynamic' AS component,
	sqlpage.run_sql('shell.sql')
	AS properties;

SELECT 'hero' AS component,
	'Presentation' AS title,
	'This sample site demonstrate a light/dark toggle' AS description;
SELECT 'text' AS component,
'Aenean pellentesque orci metus, ac imperdiet odio accumsan ac. Praesent vehicula sem lorem, in ultricies ex ultricies vitae. Nam lorem ipsum, ultrices faucibus pharetra a, maximus quis dolor. Donec malesuada, enim ut posuere pulvinar, nisl libero molestie felis, sit amet venenatis massa tortor ac enim. Etiam dui nisl, hendrerit sit amet lacinia quis, congue sed lorem. Nulla nec augue fermentum, convallis massa vel, mollis purus. Phasellus hendrerit finibus lorem vel volutpat. Cras sodales laoreet eros id consequat. Phasellus euismod ligula vitae sapien scelerisque lobortis.

Suspendisse potenti. In tempus, turpis in laoreet auctor, justo velit ullamcorper tortor, a elementum justo felis in risus. Nullam rhoncus convallis pretium. Morbi nec nisl in magna mollis ultricies quis sed tortor. Phasellus rutrum elementum vehicula. Praesent vel malesuada turpis. Vestibulum massa ante, consequat non euismod sit amet, pretium quis nisi. Nam vestibulum nulla lorem. Sed pharetra euismod eleifend. Cras ac lacus sed nunc volutpat tristique sed quis nunc.

Ut rutrum tempor orci eu fermentum. Aenean fringilla, metus a molestie blandit, velit nunc ornare ex, vel feugiat neque odio sed erat. Proin convallis, dui sit amet auctor venenatis, mauris elit hendrerit justo, sed maximus nulla orci eget felis. Praesent dolor velit, luctus et urna posuere, pulvinar dictum urna. Curabitur sed dictum felis. In at neque ornare, convallis nibh et, mollis risus. Praesent commodo vehicula dolor in egestas. Praesent euismod nunc risus, sed consequat turpis venenatis quis. Proin in risus ornare, mattis tortor sed, porttitor nunc.'
AS contents_md;
