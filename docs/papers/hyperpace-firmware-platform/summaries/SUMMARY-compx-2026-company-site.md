# SUMMARY: Compx (匠盟科技) company site

**Claim.** Compx is Shenzhen Compx Technology Co., Ltd. (深圳市匠盟科技有限公司, 匠盟), founded 2015, a Chinese fabless "solution design company" (方案设计公司) focused on short-range wireless (BT/BLE/WiFi/2.4G/5.8G), whose lead market is wireless input peripherals. It claims a self-developed 2.4G private protocol for audio.

**Method.** Fetched the site (fetch tool failed on an expired TLS cert; used curl -k) and translated. Read home, product, and news pages.

**Result.** Establishes company identity and self-description; no chip datasheets or part numbers on the site.

**Evidence tier.** 1 for company identity (own website); 4 for capability claims (marketing). Transport unauthenticated (expired cert).

**Performance.** N/A.

**Correctness.** Proven: the company exists, is the "Compx"/"匠盟" behind the HYPACE ODM strings and the export trailer "Compx Inc". Not shown: any chip part number (CX52650/N, CX52850) or bootloader detail; the site is marketing, not documentation. Note the news page hosts the reference multi-mode gaming-mouse demo driver + 4K dongle .bin analyzed separately.

**Relevance to hyperpace.** Confirms who Compx is and that public technical documentation does not exist on their site, so hyperpace's understanding must come from artifacts (the upgrade file), reverse engineering (OpenMouse), and the underlying chip vendors (WCH, Nordic, Beken).
