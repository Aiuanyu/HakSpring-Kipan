package com.siansiansu.taigikeyboard.ime.dictionary

import org.junit.Assert.assertEquals
import org.junit.Test

class ExternalLookupURLBuilderTest {

    @Test
    fun hakspringURL_generatesExpectedQueryParameters() {
        val urlSi = ExternalLookupURLBuilder.hakspringURL("便當", "xiien")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=si", urlSi)

        val urlNa = ExternalLookupURLBuilder.hakspringURL("便當", "namxiian")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=na", urlNa)

        val urlHa = ExternalLookupURLBuilder.hakspringURL("便當", "hoiliug")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=ha", urlHa)

        val urlDa = ExternalLookupURLBuilder.hakspringURL("便當", "taibu")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=da", urlDa)

        val urlRh = ExternalLookupURLBuilder.hakspringURL("便當", "ngiaupin")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=rh", urlRh)

        val urlZh = ExternalLookupURLBuilder.hakspringURL("便當", "zhioon")
        assertEquals("https://hakspring.pages.dev/?musiid=hak&ca=%E4%BE%BF%E7%95%B6&kiong=zh", urlZh)
    }
}
