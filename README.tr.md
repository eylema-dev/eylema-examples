# Eyleme örnekleri

**Tek Eyleme markası — 15 Ekim 2026:** [eyleme.dev](https://eyleme.dev/tr/), Eyleme ve topluluğunun ortak merkezidir. Eyleme bağımsız kullanılabilen, public ve Apache-2.0 lisanslı bir proje olarak kalır. **Eyleme Derleme** ve **Eyleme Onama**, isteğe bağlı ürünler olarak private kaynak kodlu, ücretli abonelik modelini izler. Ticari hizmetler ve koşullar geliştirme aşamasındadır; fiyatlar ve kullanıma sunulma durumu yayımlanmamıştır. Bu depo abonelik hizmeti sunmaz.

Ürün ailesi **Derleme → Onama → Eyleme** sırasıyla anlatılır: veriyi hazırla, işi yetkilendir, ardından yürüt. Eyleme kendi temel erişim kontrollerini korur ve bağımsız kullanım için iki aboneliğe de ihtiyaç duymaz. [Public ürün ailesi açıklamasına](https://github.com/eyleme-dev/eyleme/blob/main/docs/tr/product-family.md) bakın.

Teknik tanımlayıcılar uyumluluk için mevcut yazımlarını korur. Tarihsel kanıtlar özgün ad ve bağlantılarını korur.

Tekrarlanabilir örnek uygulamalar. [English](README.md).

**Durum: çalıştırılabilir bir dış tüketici örneği.** `task-console/`, birinci taraf çekirdeğin yerel görev hizmetine bağlanan küçük bir Rust istemcisidir; `b06b7bac1aa121a4fee81e9fd0f79e4cf2eeeb3f` çekirdek commit'ine sabitlenmiştir. Çekirdek çalışma zamanını içermez ve üçüncü taraf crate kullanmaz.

## Kapsam

Task-console örneği görev oluşturup inceler, belirlenimci bir adım gönderir ve yürütmeyi inceler. Oturum okuyucusu yalnızca özel izinli dosya ve IPv4 loopback adresini kabul eder. Varsayılan oturum insan yetkisine sahip olmadığından iptal isteğinin `FORBIDDEN` dönmesi beklenir; bu örnek insan yetkisiyle etkin durdurma iddiasında bulunmaz.

Rust birim ve dokümantasyon testleri için `cargo test --workspace --locked` çalıştırın. Biçim ve uyarıları `cargo fmt --all -- --check` ve `cargo clippy --workspace --all-targets --locked -- -D warnings` ile denetleyin. Yedi yerel kabul senaryosu için önce sabitlenmiş çekirdek CLI'ını belgelenen yerel komutla derleyin, ardından `cargo run --locked --bin acceptance -- /path/to/eylema` çalıştırın. Kabul koşucusu hizmeti başlatmadan önce verilen checkout'un `HEAD` değerini doğrular ve geçici, özel izinli veri dizinleri kullanır. F6 uyumsuz günlük fixture'ını kapsar; fiziksel güç kesintisi testi değildir.

Üretim Rust kodu, ortak tipler, çalışma zamanı/model yürütmesi ve CLI [Eyleme](https://github.com/eyleme-dev/eyleme) içinde; ürün sözleşmeleri [İngilizce](https://github.com/eyleme-dev/eyleme-docs-en) ve [Türkçe](https://github.com/eyleme-dev/eyleme-docs-tr) belge depolarındadır. [Depo haritası](https://github.com/eyleme-dev/devguide/blob/main/docs/tr/repository-map.md) sınırları tanımlar.

## Kanıt sınırı

Rust profili ve yerel kabul koşucusu bu örneğin belirtilen makinedeki derleme ve davranışını kanıtlar. GPU, model, eğitim, platformlar arası veya fiziksel güç kesintisi davranışını kanıtlamaz. Bu örnek profilde izin verilen tek harici Cargo bağımlılığı, sabitlenmiş `eylema-contracts` paketidir.

[Katkı rehberini](CONTRIBUTING.tr.md) ve [AGENTS](AGENTS.md) sözleşmesini kullanın. Özel veriler, kimlik bilgileri, operasyon kayıtları ve model ağırlıkları Git dışında kalır. Açık varlıklar için ayrıca onaylanmış depolama, köken ve lisans kararı gerekir; burada depolama veya hizmet kurulmaz. Kendi kaynak kodumuz ve belgelerimiz [Apache-2.0](LICENSE) lisanslıdır. Ayrı belirtilen üçüncü taraf içerikleri, model ağırlıkları ve veri kümeleri kendi koşullarını korur.
