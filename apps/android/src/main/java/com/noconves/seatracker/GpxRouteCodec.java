package com.noconves.seatracker;

import android.util.Xml;

import org.maplibre.android.geometry.LatLng;
import org.xmlpull.v1.XmlPullParser;
import org.xmlpull.v1.XmlSerializer;

import java.io.StringReader;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.List;

public final class GpxRouteCodec {
    public static final class Data {
        public final List<LatLng> waypoints = new ArrayList<>();
        public final List<LatLng> route = new ArrayList<>();
    }

    private GpxRouteCodec() {}

    public static Data parse(String xml) throws Exception {
        Data data = new Data();
        XmlPullParser parser = Xml.newPullParser();
        parser.setInput(new StringReader(xml));

        int event;
        while ((event = parser.next()) != XmlPullParser.END_DOCUMENT) {
            if (event != XmlPullParser.START_TAG) continue;
            String name = parser.getName();
            if (!"wpt".equals(name) && !"rtept".equals(name)) continue;

            String latRaw = parser.getAttributeValue(null, "lat");
            String lonRaw = parser.getAttributeValue(null, "lon");
            if (latRaw == null || lonRaw == null) continue;

            double lat = Double.parseDouble(latRaw);
            double lon = Double.parseDouble(lonRaw);
            if (lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0) continue;

            LatLng point = new LatLng(lat, lon);
            if ("wpt".equals(name)) data.waypoints.add(point);
            else data.route.add(point);
        }
        return data;
    }

    public static String exportRoute(String routeName, List<LatLng> route) throws Exception {
        StringWriter writer = new StringWriter();
        XmlSerializer serializer = Xml.newSerializer();
        serializer.setOutput(writer);
        serializer.startDocument("UTF-8", true);
        serializer.startTag(null, "gpx");
        serializer.attribute(null, "version", "1.1");
        serializer.attribute(null, "creator", "SeaTracker");
        serializer.attribute(null, "xmlns", "http://www.topografix.com/GPX/1/1");

        serializer.startTag(null, "rte");
        serializer.startTag(null, "name");
        serializer.text(routeName == null || routeName.isEmpty() ? "SeaTracker Route" : routeName);
        serializer.endTag(null, "name");

        int index = 1;
        for (LatLng point : route) {
            serializer.startTag(null, "rtept");
            serializer.attribute(null, "lat", Double.toString(point.getLatitude()));
            serializer.attribute(null, "lon", Double.toString(point.getLongitude()));
            serializer.startTag(null, "name");
            serializer.text("WP" + index++);
            serializer.endTag(null, "name");
            serializer.endTag(null, "rtept");
        }

        serializer.endTag(null, "rte");
        serializer.endTag(null, "gpx");
        serializer.endDocument();
        return writer.toString();
    }
}
