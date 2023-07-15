-- Insert topics
INSERT INTO topic (name, icon) VALUES
    ('Technology', 'photo-code'),
    ('Travel', 'plane'),
    ('Food', 'pizza'),
    ('Health', 'heartbeat'),
    ('Entertainment', 'player-track-next-filled'),
    ('Sports', 'ball-football'),
    ('Nature', 'tree'),
    ('Fashion', 'tie'),
    ('Finance', 'cash'),
    ('Education', 'book'),
    ('Science', 'flask'),
    ('Music', 'headphones'),
    ('Art', 'palette'),
    ('Fitness', 'barbell'),
    ('Politics', 'globe'),
    ('History', 'building-arch'),
    ('Lifestyle', 'camera'),
    ('Gaming', 'brand-apple-arcade'),
    ('Business', 'briefcase'),
    ('Psychology', 'brain');

-- Insert posts
-- Post 1
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    'The Future of Artificial Intelligence',
    '# The Future of Artificial Intelligence

Artificial Intelligence (AI) has been a topic of great interest and speculation in recent years. With advancements in technology and the increasing availability of data, AI has the potential to revolutionize various industries and aspects of our lives.

One of the key areas where AI is making significant strides is in machine learning. Machine learning algorithms enable computers to analyze large amounts of data and learn patterns and trends from it. This has applications in fields such as finance, healthcare, and marketing, among others.

Another exciting development in AI is the emergence of deep learning. Deep learning is a subfield of machine learning that focuses on neural networks and their ability to learn and make decisions in a way similar to the human brain. Deep learning has shown promising results in image and speech recognition, natural language processing, and autonomous driving.

While AI presents tremendous opportunities, it also raises important ethical and societal questions. Concerns about job displacement, privacy, and biases in AI algorithms need to be addressed to ensure responsible and beneficial use of AI.

As we look ahead, it''s clear that AI will continue to shape the future. It has the potential to drive innovation, improve efficiency, and solve complex problems. However, it''s essential to strike a balance between technological advancement and ethical considerations to harness the full potential of AI for the benefit of society.',
    '2022-08-10 12:00:00',
    (SELECT id FROM topic WHERE name = 'Technology')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'Business'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Politics'), last_insert_rowid());

-- Post 2
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    'Exploring the Hidden Gems of Europe',
    '# Exploring the Hidden Gems of Europe

Europe is a continent known for its rich history, diverse cultures, and breathtaking landscapes. While popular destinations like Paris, Rome, and Barcelona attract millions of tourists each year, there are also lesser-known places that offer unique experiences and a chance to discover the hidden gems of Europe.

One such hidden gem is the village of Hallstatt in Austria. Nestled between the mountains and the Hallstätter See, this picturesque village is like something out of a fairytale. Its charming streets, traditional wooden houses, and stunning views make it a perfect destination for nature lovers and photographers.

Another hidden gem is the Plitvice Lakes National Park in Croatia. With its cascading waterfalls, crystal clear lakes, and lush greenery, this national park is a paradise for hikers and nature enthusiasts. Exploring the park''s network of wooden walkways and taking a boat ride on the lakes is an unforgettable experience.

In Portugal, the village of Monsaraz is a hidden gem that offers a glimpse into the country''s medieval past. Perched on a hilltop, the village is surrounded by fortified walls and offers panoramic views of the surrounding countryside. Its narrow streets, whitewashed houses, and historic castle create a magical atmosphere.

These are just a few examples of the hidden gems that Europe has to offer. Exploring off-the-beaten-path destinations can be a rewarding experience, allowing you to discover the lesser-known treasures and immerse yourself in the local culture.

So, the next time you plan a trip to Europe, consider venturing beyond the popular tourist destinations and embark on a journey to explore the hidden gems that await you.',
    '2023-04-22 10:30:00',
    (SELECT id FROM topic WHERE name = 'Travel')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'History'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Art'), last_insert_rowid());

-- Post 3
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    'Delicious and Healthy Recipes for Every Foodie',
    '# Delicious and Healthy Recipes for Every Foodie

Are you a food lover who enjoys both delicious flavors and maintaining a healthy lifestyle? If so, you''re in luck! In this post, we''ll share some mouthwatering recipes that are not only incredibly tasty but also packed with nutritious ingredients.

**1. Quinoa Salad with Roasted Vegetables**

Ingredients:
- 1 cup quinoa
- Assorted vegetables (such as bell peppers, zucchini, and cherry tomatoes)
- Olive oil
- Balsamic vinegar
- Salt and pepper
- Fresh herbs (such as parsley or basil)

Instructions:
- Cook the quinoa according to package instructions.
- Preheat the oven to 400°F (200°C).
- Chop the vegetables into bite-sized pieces and place them on a baking sheet.
- Drizzle with olive oil, balsamic vinegar, salt, and pepper.
- Roast in the oven for 20-25 minutes, or until the vegetables are tender and slightly charred.
- In a large bowl, combine the cooked quinoa and roasted vegetables.
- Drizzle with additional olive oil and vinegar if desired.
- Season with salt, pepper, and fresh herbs.

**2. Grilled Salmon with Lemon and Dill**

Ingredients:
- Salmon fillets
- Lemon slices
- Fresh dill
- Salt and pepper
- Olive oil

Instructions:
- Preheat the grill to medium-high heat.
- Season the salmon fillets with salt, pepper, and a drizzle of olive oil.
- Place the salmon on the grill, skin-side down.
- Grill for about 4-5 minutes on each side, or until cooked to your desired level of doneness.
- During the last minute of grilling, place a few lemon slices and fresh dill on top of each salmon fillet.
- Remove from the grill and serve immediately.

**3. Mango and Avocado Salsa**

Ingredients:
- Ripe mango, diced
- Ripe avocado, diced
- Red onion, finely chopped
- Fresh cilantro, chopped
- Lime juice
- Salt and pepper

Instructions:
- In a bowl, combine the diced mango, avocado, red onion, and cilantro.
- Squeeze fresh lime juice over the mixture and gently toss to combine.
- Season with salt and pepper to taste.
- Let the salsa sit for a few minutes to allow the flavors to meld together.
- Serve with tortilla chips or as a topping for grilled chicken or fish.

These recipes are just a starting point for your culinary adventures. Feel free to modify them to suit your taste preferences and experiment with different ingredients. Enjoy the delicious and healthy creations you make in your kitchen!

**Note**: Always consult your doctor or a qualified nutritionist before making any significant changes to your diet or if you have any specific dietary concerns or restrictions.',
    '2022-12-05 15:15:00',
    (SELECT id FROM topic WHERE name = 'Food')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'Food'), last_insert_rowid());

-- Post 4
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    '10 Tips for a Healthier Lifestyle',
    '# 10 Tips for a Healthier Lifestyle

Maintaining a healthy lifestyle is essential for overall well-being and longevity. Small changes in your daily habits can make a significant difference in improving your health. Here are ten tips to help you lead a healthier life:

**1. Eat a Balanced Diet**: Focus on consuming a variety of whole foods, including fruits, vegetables, lean proteins, whole grains, and healthy fats. Limit processed foods, sugary drinks, and excessive salt intake.

**2. Stay Hydrated**: Drink plenty of water throughout the day to keep your body hydrated. Avoid sugary beverages and excessive alcohol consumption.

**3. Get Regular Exercise**: Aim for at least 150 minutes of moderate-intensity aerobic activity or 75 minutes of vigorous-intensity aerobic activity each week. Include strength training exercises to build muscle and improve bone density.

**4. Prioritize Sleep**: Make sleep a priority and aim for 7-9 hours of quality sleep each night. Establish a consistent sleep schedule and create a relaxing bedtime routine.

**5. Manage Stress**: Find healthy ways to manage stress, such as practicing mindfulness meditation, deep breathing exercises, or engaging in hobbies that bring you joy.

**6. Maintain a Healthy Weight**: Strive to achieve and maintain a healthy weight for your body. This can be accomplished through a combination of healthy eating and regular physical activity.

**7. Practice Good Hygiene**: Follow proper hygiene practices, including regular handwashing, dental care, and getting vaccinated as recommended by healthcare professionals.

**8. Limit Screen Time**: Reduce the amount of time spent in front of screens, including smartphones, tablets, and computers. Take regular breaks and engage in physical activities or social interactions.

**9. Foster Healthy Relationships**: Cultivate positive relationships with family, friends, and a supportive community. Social connections contribute to overall well-being.

**10. Take Time for Self-Care**: Prioritize self-care activities that promote relaxation, such as taking a bath, reading a book, or engaging in hobbies you enjoy.

Remember, it''s essential to consult with healthcare professionals for personalized advice and guidance. By incorporating these tips into your daily routine, you can embark on a journey towards a healthier and more fulfilling life.

**Note**: The information provided is for general educational purposes and should not replace professional medical advice.',
    '2023-01-20 09:45:00',
    (SELECT id FROM topic WHERE name = 'Health')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'Fitness'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Health'), last_insert_rowid());

-- Post 5
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    'The Rise of Streaming Platforms',
    '# The Rise of Streaming Platforms

Streaming platforms have revolutionized the way we consume media and entertainment. With the advent of high-speed internet and advancements in technology, streaming has become increasingly popular and has disrupted traditional media channels.

Gone are the days of waiting for your favorite TV show to air on a specific day and time. Streaming platforms offer on-demand access to a vast library of movies, TV shows, documentaries, and original content. Whether it''s Netflix, Amazon Prime Video, Disney+, or other streaming services, there''s something for everyone.

One of the key advantages of streaming platforms is the ability to watch content anytime, anywhere, and on any device. With a stable internet connection, you can stream your favorite shows on your TV, computer, tablet, or smartphone. This convenience has transformed the way we consume entertainment, giving us more control over our viewing experience.

Streaming platforms have also opened doors for diverse storytelling and content creation. They provide a platform for independent filmmakers, content creators, and artists to showcase their work and reach a global audience. This has led to the production of unique and innovative content that may not have found a place in traditional media channels.

However, the rise of streaming platforms has not been without challenges. Issues such as content licensing, regional restrictions, and subscription costs have raised concerns among consumers. Additionally, the abundance of content can sometimes be overwhelming, making it difficult to choose what to watch.

As streaming platforms continue to evolve, we can expect further advancements in technology, improved user experiences, and a more competitive landscape. The future of entertainment lies in the hands of streaming, and it''s an exciting time for both creators and consumers.

So, grab your popcorn, find a comfortable spot on the couch, and immerse yourself in the world of streaming entertainment!',
    '2022-09-17 18:20:00',
    (SELECT id FROM topic WHERE name = 'Entertainment')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'Gaming'), last_insert_rowid());

-- Post 6
INSERT INTO post (title, content, created_at, main_topic_id) VALUES (
    'The Thrilling World of Extreme Sports',
    '# The Thrilling World of Extreme Sports

If you are an adrenaline junkie and seek thrilling experiences, extreme sports might be your calling. These activities push the boundaries of what''s possible and offer an exhilarating rush that can''s be matched by anything else.

**1. Skydiving**: Jumping out of a plane and freefalling through the sky is the epitome of an adrenaline rush. Whether you''re a first-time skydiver or an experienced jumper, the feeling of soaring through the air is unparalleled.

**2. Bungee Jumping**: Plunging from a high platform or bridge with only a bungee cord attached to your feet is an extreme sport that requires courage and a love for heights. The thrill of the freefall and the rebounding sensation as the cord pulls you back up is an experience like no other.

**3. Rock Climbing**: Scaling cliffs, mountains, or indoor walls requires physical strength, mental focus, and problem-solving skills. The sense of achievement when reaching the top and enjoying the breathtaking views is incredibly rewarding.

**4. Whitewater Rafting**: Battling through turbulent rapids and navigating rivers is an adrenaline-pumping adventure. Paddling as a team and overcoming the challenges of the water adds an element of excitement and camaraderie.

**5. Snowboarding**: Gliding down snow-covered slopes, performing tricks, and feeling the rush of speed is what snowboarding is all about. Whether you prefer freestyle or backcountry riding, the mountains offer endless possibilities for adrenaline-fueled fun.

**6. Surfing**: Riding the waves and harnessing the power of the ocean is a thrilling experience for surfers. Whether you''re a beginner catching your first wave or an expert riding massive swells, the feeling of being in sync with the water is pure bliss.

**7. Base Jumping**: BASE stands for Building, Antenna, Span, and Earth, representing the four types of fixed objects from which BASE jumpers leap. It''s an extreme sport that involves parachuting from fixed objects and requires specialized skills and training.

These are just a few examples of extreme sports that offer excitement and adventure. However, it''s important to remember that these activities come with inherent risks, and proper training, equipment, and safety precautions should always be prioritized.

If you''re craving an adrenaline rush and are willing to step outside your comfort zone, give extreme sports a try. Just be prepared to experience the thrill of a lifetime!',
    '2023-06-07 14:10:00',
    (SELECT id FROM topic WHERE name = 'Sports')
);
INSERT INTO topic_post (topic_id, post_id) VALUES
    ((SELECT id FROM topic WHERE name = 'Fitness'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Entertainment'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Health'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Lifestyle'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Travel'), last_insert_rowid()),
    ((SELECT id FROM topic WHERE name = 'Nature'), last_insert_rowid());
