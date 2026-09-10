package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileReader;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * CM93/2 compatibility decoder. Input is treated as untrusted; all sizes and
 * references are validated before allocation/use.
 */
public final class Cm93ChartDecoder {
    private static final double RADIUS_M = 6_378_388.0;
    private static final int MAX_POINTS = 8_000_000;
    private static final int MAX_FEATURES = 500_000;
    private static final int MAX_RENDER_FEATURES = 20_000;
    private static final int[] TABLE = {
        0xCD,0xEA,0xDC,0x48,0x3E,0x6D,0xCA,0x7B,0x52,0xE1,0xA4,0x8E,0xAB,0x05,0xA7,0x97,
        0xB9,0x60,0x39,0x85,0x7C,0x56,0x7A,0xBA,0x68,0x6E,0xF5,0x5D,0x02,0x4E,0x0F,0xA1,
        0x27,0x24,0x41,0x34,0x00,0x5A,0xFE,0xCB,0xD0,0xFA,0xF8,0x6C,0x74,0x96,0x9E,0x0E,
        0xC2,0x49,0xE3,0xE5,0xC0,0x3B,0x59,0x18,0xA9,0x86,0x8F,0x30,0xC3,0xA8,0x22,0x0A,
        0x14,0x1A,0xB2,0xC9,0xC7,0xED,0xAA,0x29,0x94,0x75,0x0D,0xAC,0x0C,0xF4,0xBB,0xC5,
        0x3F,0xFD,0xD9,0x9C,0x4F,0xD5,0x84,0x1E,0xB1,0x81,0x69,0xB4,0x09,0xB8,0x3C,0xAF,
        0xA3,0x08,0xBF,0xE0,0x9A,0xD7,0xF7,0x8C,0x67,0x66,0xAE,0xD4,0x4C,0xA5,0xEC,0xF9,
        0xB6,0x64,0x78,0x06,0x5B,0x9B,0xF2,0x99,0xCE,0xDB,0x53,0x55,0x65,0x8D,0x07,0x33,
        0x04,0x37,0x92,0x26,0x23,0xB5,0x58,0xDA,0x2F,0xB3,0x40,0x5E,0x7F,0x4B,0x62,0x80,
        0xE4,0x6F,0x73,0x1D,0xDF,0x17,0xCC,0x28,0x25,0x2D,0xEE,0x3A,0x98,0xE2,0x01,0xEB,
        0xDD,0xBC,0x90,0xB0,0xFC,0x95,0x76,0x93,0x46,0x57,0x2C,0x2B,0x50,0x11,0x0B,0xC1,
        0xF0,0xE7,0xD6,0x21,0x31,0xDE,0xFF,0xD8,0x12,0xA6,0x4D,0x8A,0x13,0x43,0x45,0x38,
        0xD2,0x87,0xA0,0xEF,0x82,0xF1,0x47,0x89,0x6A,0xC8,0x54,0x1B,0x16,0x7E,0x79,0xBD,
        0x6B,0x91,0xA2,0x71,0x36,0xB7,0x03,0x3D,0x72,0xC6,0x44,0x8B,0xCF,0x15,0x9F,0x32,
        0xC4,0x77,0x83,0x63,0x20,0x88,0xF6,0xAD,0xF3,0xE8,0x4A,0xE9,0x35,0x1C,0x5F,0x19,
        0x1F,0x7D,0x70,0xFB,0xD1,0x51,0x10,0xD3,0x2E,0x61,0x9D,0x5C,0x2A,0x42,0xBE,0xE6
    };
    private static final int[] DECODE = buildDecode();

    public enum GeometryType { POINT, LINE, AREA, SOUNDINGS, NONE }

    public static final class Feature {
        public final int objectType;
        public final String className;
        public final GeometryType type;
        public final List<LatLng> points;
        public final List<Double> depths;

        Feature(int objectType, String className, GeometryType type, List<LatLng> points, List<Double> depths) {
            this.objectType = objectType;
            this.className = className;
            this.type = type;
            this.points = points;
            this.depths = depths;
        }
    }

    public static final class Result {
        public final File file;
        public final double minLat, minLon, maxLat, maxLon;
        public final int scale;
        public final int rawFeatureCount;
        public final List<Feature> features;

        Result(File file, double minLat, double minLon, double maxLat, double maxLon, int scale, int rawFeatureCount, List<Feature> features) {
            this.file = file;
            this.minLat = minLat;
            this.minLon = minLon;
            this.maxLat = maxLat;
            this.maxLon = maxLon;
            this.scale = scale;
            this.rawFeatureCount = rawFeatureCount;
            this.features = features;
        }

        public String summary() {
            return file.getName() + " • 1:" + scale + " • " + rawFeatureCount + " objetos";
        }
    }

    private static final class Header {
        double lonMin, latMin, lonMax, latMax;
        double eastingMin, northingMin, eastingMax, northingMax;
        int vectorRecords, vectorPoints, point3dRecords, point3dPoints, point2dRecords, featureRecords;
    }

    private static final class P { int x, y; P(int x, int y) { this.x=x; this.y=y; } }
    private static final class P3 { int x, y, z; P3(int x,int y,int z){this.x=x;this.y=y;this.z=z;} }
    private static final class EdgeRef { int index, usage; EdgeRef(int index,int usage){this.index=index;this.usage=usage;} }

    private static final class RawFeature {
        int objectType, flags;
        int pointIndex = -1, soundingIndex = -1;
        List<EdgeRef> edges = Collections.emptyList();
    }

    private static final class Reader {
        final byte[] bytes; int pos;
        Reader(byte[] bytes) { this.bytes=bytes; }
        void require(int n) throws Exception { if (n < 0 || pos > bytes.length - n) throw new Exception("CM93 truncado"); }
        byte[] decoded(int n) throws Exception { require(n); byte[] out=new byte[n]; for(int i=0;i<n;i++) out[i]=(byte)DECODE[bytes[pos+i]&255]; pos+=n; return out; }
        int u8() throws Exception { return decoded(1)[0]&255; }
        int u16() throws Exception { byte[] b=decoded(2); return (b[0]&255)|((b[1]&255)<<8); }
        int i32() throws Exception { byte[] b=decoded(4); return ByteBuffer.wrap(b).order(ByteOrder.LITTLE_ENDIAN).getInt(); }
        double f64() throws Exception { byte[] b=decoded(8); return ByteBuffer.wrap(b).order(ByteOrder.LITTLE_ENDIAN).getDouble(); }
        void seek(int p) throws Exception { if(p<0||p>bytes.length)throw new Exception("Offset CM93 inválido"); pos=p; }
    }

    private Cm93ChartDecoder() {}

    public static boolean isCm93CellName(String name) {
        if (name == null) return false;
        String n = name.toUpperCase(Locale.ROOT);
        if (n.endsWith(".XZ")) n = n.substring(0, n.length()-3);
        int dot=n.lastIndexOf('.');
        if(dot<0||dot!=n.length()-2)return false;
        char scale=n.charAt(n.length()-1);
        if("ZABCDEFG".indexOf(scale)<0)return false;
        String stem=n.substring(0,dot);
        return stem.length()==8 && (Character.isLetterOrDigit(stem.charAt(0)));
    }

    public static int scaleForFileName(String name) {
        if(name==null)return 20_000_000;
        String n=name.toUpperCase(Locale.ROOT);
        if(n.endsWith(".XZ"))n=n.substring(0,n.length()-3);
        char c=n.charAt(n.length()-1);
        switch(c){case 'A':return 3_000_000;case 'B':return 1_000_000;case 'C':return 200_000;case 'D':return 100_000;case 'E':return 50_000;case 'F':return 20_000;case 'G':return 7_500;default:return 20_000_000;}
    }

    public static Map<Integer,String> loadObjectDictionary(File directory) {
        Map<Integer,String> out=new HashMap<>();
        File dictionary=findIgnoreCase(directory,"CM93OBJ.DIC");
        if(dictionary==null)return out;
        try(BufferedReader reader=new BufferedReader(new FileReader(dictionary))){
            String line;
            while((line=reader.readLine())!=null){
                if(line.trim().isEmpty()||line.startsWith(";"))continue;
                String[] fields=line.split("\\|");
                if(fields.length<2)continue;
                try{out.put(Integer.parseInt(fields[1].trim()),fields[0].trim());}catch(NumberFormatException ignored){}
            }
        }catch(Exception ignored){}
        return out;
    }

    private static File findIgnoreCase(File root,String name){
        if(root==null||!root.exists())return null;
        File[] files=root.listFiles(); if(files==null)return null;
        for(File f:files)if(f.isFile()&&f.getName().equalsIgnoreCase(name))return f;
        for(File f:files)if(f.isDirectory()){File found=findIgnoreCase(f,name);if(found!=null)return found;}
        return null;
    }

    public static Result decode(File file, Map<Integer,String> dictionary) throws Exception {
        long len=file.length(); if(len<16||len>256L*1024L*1024L)throw new Exception("Tamanho de célula CM93 inválido");
        byte[] bytes=Cm93FileBytes.read(file);
        if(bytes.length<138)throw new Exception("CM93 descompactado é muito curto");
        Reader r=new Reader(bytes);
        int prolog=r.u16(); int t1=r.i32(); int t2=r.i32();
        if(prolog<138||t1<0||t2<0||((long)prolog+t1+t2)!=bytes.length)throw new Exception("Prólogo CM93 inválido");
        Header h=readHeader(r); if(r.pos!=138)throw new Exception("Header CM93 inválido");

        List<List<P>> edges=new ArrayList<>(h.vectorRecords); int edgePointCount=0;
        for(int i=0;i<h.vectorRecords;i++){
            int count=r.u16(); edgePointCount=safeAdd(edgePointCount,count,"vetores"); if(edgePointCount>MAX_POINTS)throw new Exception("CM93 excede limite de vetores");
            List<P> edge=new ArrayList<>(count); for(int j=0;j<count;j++)edge.add(new P(r.u16(),r.u16())); edges.add(edge);
        }
        if(h.vectorPoints>0&&edgePointCount!=h.vectorPoints)throw new Exception("Contagem de vetores CM93 inconsistente");

        List<List<P3>> soundings=new ArrayList<>(h.point3dRecords); int p3count=0;
        for(int i=0;i<h.point3dRecords;i++){
            int count=r.u16(); p3count=safeAdd(p3count,count,"sondagens"); if(p3count>MAX_POINTS)throw new Exception("CM93 excede limite de sondagens");
            List<P3> record=new ArrayList<>(count); for(int j=0;j<count;j++)record.add(new P3(r.u16(),r.u16(),r.u16())); soundings.add(record);
        }
        if(h.point3dPoints>0&&p3count!=h.point3dPoints)throw new Exception("Contagem 3D CM93 inconsistente");

        List<P> points=new ArrayList<>(h.point2dRecords); for(int i=0;i<h.point2dRecords;i++)points.add(new P(r.u16(),r.u16()));
        List<RawFeature> raw=new ArrayList<>(h.featureRecords);
        for(int i=0;i<h.featureRecords;i++){
            int start=r.pos; RawFeature f=new RawFeature(); f.objectType=r.u8(); f.flags=r.u8(); int recordLen=r.u16();
            if(recordLen<4)throw new Exception("Registro CM93 inválido"); int end=safeAdd(start,recordLen,"registro"); if(end>bytes.length)throw new Exception("Registro CM93 truncado");
            int primitive=f.flags&0x0f;
            if(primitive==1){f.pointIndex=r.u16();if(f.pointIndex>=points.size())throw new Exception("Referência de ponto CM93 inválida");}
            else if(primitive==2||primitive==4){int count=r.u16();List<EdgeRef> refs=new ArrayList<>(count);for(int j=0;j<count;j++){int packed=r.u16();int idx=packed&0x1fff;if(idx>=edges.size())throw new Exception("Referência de edge CM93 inválida");refs.add(new EdgeRef(idx,packed>>13));}f.edges=refs;}
            else if(primitive==8){f.soundingIndex=r.u16();if(f.soundingIndex>=soundings.size())throw new Exception("Referência 3D CM93 inválida");}
            if((f.flags&0x10)!=0){int count=r.u8();for(int j=0;j<count;j++)r.u16();}
            if((f.flags&0x20)!=0)r.u16();
            if((f.flags&0x80)!=0){r.u8(); if(r.pos>end)throw new Exception("Atributos CM93 inválidos"); r.decoded(end-r.pos);}
            if(r.pos>end)throw new Exception("Registro CM93 excedeu seu limite"); if(r.pos<end)r.seek(end); raw.add(f);
        }

        List<Feature> features=new ArrayList<>(); int rendered=0;
        for(RawFeature f:raw){
            if(rendered>=MAX_RENDER_FEATURES)break;
            String className=dictionary.getOrDefault(f.objectType,"CM93_"+f.objectType);
            int primitive=f.flags&0x0f;
            if(primitive==1){P p=points.get(f.pointIndex);features.add(new Feature(f.objectType,className,GeometryType.POINT,Collections.singletonList(transform(h,p)),Collections.emptyList()));rendered++;}
            else if(primitive==2||primitive==4){List<LatLng> ll=assemble(h,edges,f.edges);if(ll.size()>=2){features.add(new Feature(f.objectType,className,primitive==4?GeometryType.AREA:GeometryType.LINE,ll,Collections.emptyList()));rendered++;}}
            else if(primitive==8){List<P3> rec=soundings.get(f.soundingIndex);List<LatLng> ll=new ArrayList<>(rec.size());List<Double> depths=new ArrayList<>(rec.size());for(P3 p:rec){ll.add(transform(h,new P(p.x,p.y)));depths.add(p.z>=12000?(double)(p.z-12000):p.z/10.0);}features.add(new Feature(f.objectType,className,GeometryType.SOUNDINGS,ll,depths));rendered++;}
        }
        return new Result(file,h.latMin,normalizeLon(h.lonMin),h.latMax,normalizeLon(h.lonMax),scaleForFileName(file.getName()),h.featureRecords,features);
    }

    private static Header readHeader(Reader r)throws Exception{
        Header h=new Header(); h.lonMin=r.f64();h.latMin=r.f64();h.lonMax=r.f64();h.latMax=r.f64();h.eastingMin=r.f64();h.northingMin=r.f64();h.eastingMax=r.f64();h.northingMax=r.f64();
        double[] vals={h.lonMin,h.latMin,h.lonMax,h.latMax,h.eastingMin,h.northingMin,h.eastingMax,h.northingMax};for(double v:vals)if(!Double.isFinite(v))throw new Exception("Header CM93 não finito");
        h.vectorRecords=r.u16();h.vectorPoints=nonnegative(r.i32(),"vectorPoints");r.i32();r.i32();h.point3dRecords=r.u16();h.point3dPoints=nonnegative(r.i32(),"point3dPoints");r.i32();h.point2dRecords=r.u16();r.u16();r.u16();h.featureRecords=r.u16();r.i32();r.i32();r.u16();r.u16();r.u16();nonnegative(r.i32(),"related");r.i32();r.u16();nonnegative(r.i32(),"attributes");r.i32();
        if(h.vectorPoints>MAX_POINTS||h.point3dPoints>MAX_POINTS||h.point2dRecords>MAX_POINTS||h.featureRecords>MAX_FEATURES)throw new Exception("Header CM93 excede limites de segurança"); return h;
    }

    private static List<LatLng> assemble(Header h,List<List<P>> edges,List<EdgeRef> refs){
        List<LatLng> out=new ArrayList<>(); for(EdgeRef ref:refs){List<P> edge=edges.get(ref.index);if((ref.usage&4)==4){for(int i=edge.size()-1;i>=0;i--)appendUnique(out,transform(h,edge.get(i)));}else{for(P p:edge)appendUnique(out,transform(h,p));}}return out;
    }
    private static void appendUnique(List<LatLng> out,LatLng p){if(out.isEmpty()||out.get(out.size()-1).getLatitude()!=p.getLatitude()||out.get(out.size()-1).getLongitude()!=p.getLongitude())out.add(p);}
    private static LatLng transform(Header h,P p){double dx=h.eastingMax-h.eastingMin;if(dx<0)dx+=RADIUS_M*2.0*Math.PI;double e=p.x*(dx/65535.0)+h.eastingMin;double n=p.y*((h.northingMax-h.northingMin)/65535.0)+h.northingMin;double lat=Math.toDegrees(2.0*Math.atan(Math.exp(n/RADIUS_M))-Math.PI/2.0);double lon=normalizeLon(Math.toDegrees(e/RADIUS_M));return new LatLng(lat,lon);}
    private static double normalizeLon(double lon){double v=(lon+180.0)%360.0;if(v<0)v+=360.0;return v-180.0;}
    private static int nonnegative(int v,String field)throws Exception{if(v<0)throw new Exception("Campo CM93 negativo: "+field);return v;}
    private static int safeAdd(int a,int b,String field)throws Exception{long v=(long)a+b;if(v>Integer.MAX_VALUE)throw new Exception("Overflow CM93: "+field);return(int)v;}
    private static int[] buildDecode(){int[] out=new int[256];boolean[] seen=new boolean[256];for(int plain=0;plain<256;plain++){int encoded=TABLE[plain]^8;if(seen[encoded])throw new IllegalStateException("Tabela CM93 inválida");seen[encoded]=true;out[encoded]=plain;}return out;}
}
