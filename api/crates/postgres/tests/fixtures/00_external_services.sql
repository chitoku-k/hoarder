INSERT INTO "external_services"
    ("id", "slug", "kind", "name", "base_url", "url_pattern")
VALUES
    ('4e0c68c7-e5ec-4d60-b9eb-733f47290cd3', 'pixiv', 'pixiv', 'pixiv', 'https://www.pixiv.net', '^https?://www\.pixiv\.net/(?:artworks/|member_illust\.php\?(?:|.+&)illust_id=)(?<id>\d+)(?:[?&#].*)?$'),
    ('2018afa2-aed9-46de-af9e-02e5fab64ed7', 'skeb', 'skeb', 'Skeb', 'https://skeb.jp', '^https?://skeb\.jp/@(?<creatorId>[^/]+)/works/(?<id>\d+)(?:[?#].*)?$'),
    ('99a9f0e8-1097-4b7f-94f2-2a7d2cc786ab', 'x', 'x', 'X', 'https://x.com', '^https?://(?:twitter\.com|x\.com)/(?<creatorId>[^/]+)/status/(?<id>\d+)(?:[/?#].*)?$'),
    ('6c07eb4d-93a1-4efd-afce-e13f8f2c0e14', 'whatever', 'custom', 'Custom', NULL, NULL);
