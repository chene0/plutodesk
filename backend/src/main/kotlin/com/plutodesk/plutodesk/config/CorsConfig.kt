package com.plutodesk.plutodesk.config

import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration
import org.springframework.web.cors.CorsConfiguration
import org.springframework.web.cors.CorsConfigurationSource
import org.springframework.web.cors.UrlBasedCorsConfigurationSource

@Configuration
class CorsConfig {

    @Bean
    fun corsConfigurationSource(): CorsConfigurationSource {
        val configuration = CorsConfiguration()
        
        // Allow requests from Next.js development server and Tauri app
        configuration.allowedOrigins = listOf(
            "http://localhost:3000",           // Next.js dev server
            "https://tauri.localhost",         // Tauri app
            "tauri://localhost",               // Tauri protocol
            "http://localhost:1420",           // Tauri dev server (default port)
            "https://localhost:1420"           // Tauri dev server (HTTPS)
        )
        
        // Allow all HTTP methods
        configuration.allowedMethods = listOf("GET", "POST", "PUT", "DELETE", "OPTIONS")
        
        // Allow all headers
        configuration.allowedHeaders = listOf("*")
        
        // Allow credentials (cookies, authorization headers)
        configuration.allowCredentials = true
        
        val source = UrlBasedCorsConfigurationSource()
        source.registerCorsConfiguration("/**", configuration)
        return source
    }
}
