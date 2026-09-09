plugins { id("com.android.application") }

android {
    namespace = "com.noconves.seatracker"
    compileSdk = 36
    defaultConfig {
        applicationId = "com.noconves.seatracker"
        minSdk = 26
        targetSdk = 36
        versionCode = 2
        versionName = "0.2.0-functional"
    }
    buildTypes { release { isMinifyEnabled = false } }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
dependencies {
    implementation("androidx.appcompat:appcompat:1.8.0")
    implementation("androidx.core:core:1.17.0")
    implementation("org.maplibre.gl:android-sdk-opengl:11.8.0")
}
